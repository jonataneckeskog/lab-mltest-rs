use crate::neural::{Agent, SharedBanks, byte_stack::ByteStack, opcode::op};

pub struct AgentExecutor<'a> {
    agent: &'a mut Agent,
    shared: &'a mut SharedBanks,
    op_costs: &'a [f32; 256],
}

pub struct ExecutionSummary {
    pub reason: TerminationReason,
    pub syscalls: Vec<SysCall>,
}

pub enum TerminationReason {
    Halted { confidence: u8 },
    TimedOut,
    Died,
}

pub enum SysCall {
    SpawnChild { genome: Vec<u8>, energy: f32 },
    LeaveCommunity { key: u8 },
}

impl<'a> AgentExecutor<'a> {
    pub fn new(
        agent: &'a mut Agent,
        shared: &'a mut SharedBanks,
        op_costs: &'a [f32; 256],
    ) -> Self {
        Self {
            agent,
            shared,
            op_costs,
        }
    }

    pub fn run(&mut self, max_steps: usize) -> ExecutionSummary {
        let mut pc = 0;
        let mut nbr_executed = 0;
        let mut stack = ByteStack::new();
        let mut call_stack: Vec<usize> = Vec::with_capacity(8);
        let mut syscalls: Vec<SysCall> = Vec::new();

        let bank_ptrs: [*const [u8; 256]; 8] = [
            &self.agent.private_banks.0[0],
            &self.agent.private_banks.0[1],
            &self.agent.private_banks.0[2],
            &self.agent.private_banks.0[3],
            &self.agent.private_banks.0[4],
            &self.agent.private_banks.0[5],
            &self.shared.0[0],
            &self.shared.0[1],
        ];

        let genome_mask = self.agent.genome.len().saturating_sub(1);
        let wrap_pc = |pos: isize| -> usize { (pos as usize) & genome_mask };

        while nbr_executed < max_steps && pc < self.agent.genome.len() && self.agent.energy.0 > 0.0
        {
            let mut instruction = self.agent.genome[pc];
            pc += 1;

            loop {
                match instruction {
                    op::NO_OP => {}
                    op::HALT => {
                        let confidence = stack.pop();
                        return ExecutionSummary {
                            reason: TerminationReason::Halted { confidence },
                            syscalls,
                        };
                    }
                    op::POP => {
                        stack.pop();
                    }
                    op::DUP => {
                        stack.push(stack.peek());
                    }
                    op::SWAP => {
                        stack.swap();
                    }
                    op::PUSH => {
                        let value = self.agent.genome[pc];
                        stack.push(value);
                        pc += 1;
                    }
                    op::OVER => {
                        stack.over();
                    }
                    op::SELECT => {
                        let cond = stack.pop();
                        let val_b = stack.pop();
                        let val_a = stack.pop();
                        stack.push(if cond != 0 { val_a } else { val_b });
                    }
                    op::EXEC_STACK => {
                        self.agent.energy -= self.op_costs[op::EXEC_STACK as usize];
                        instruction = stack.pop();
                        continue;
                    }
                    op::ADD => {
                        let b = stack.pop();
                        let a = stack.pop();
                        stack.push(a.wrapping_add(b));
                    }
                    op::SUB => {
                        let b = stack.pop();
                        let a = stack.pop();
                        stack.push(a.wrapping_sub(b));
                    }
                    op::XOR => {
                        let b = stack.pop();
                        let a = stack.pop();
                        stack.push(a ^ b);
                    }
                    op::AND => {
                        let b = stack.pop();
                        let a = stack.pop();
                        stack.push(a & b);
                    }
                    op::OR => {
                        let b = stack.pop();
                        let a = stack.pop();
                        stack.push(a | b);
                    }
                    op::NOT => {
                        let a = stack.pop();
                        stack.push(!a);
                    }
                    op::SHL => {
                        let amt = stack.pop();
                        let val = stack.pop();
                        stack.push(val.wrapping_shl(amt as u32));
                    }
                    op::SHR => {
                        let amt = stack.pop();
                        let val = stack.pop();
                        stack.push(val.wrapping_shr(amt as u32));
                    }
                    op::MUL => {
                        let b = stack.pop();
                        let a = stack.pop();
                        stack.push(a.wrapping_mul(b));
                    }
                    op::DIV => {
                        let b = stack.pop();
                        let a = stack.pop();
                        stack.push(if b == 0 { 0 } else { a.wrapping_div(b) });
                    }
                    op::MOD => {
                        let b = stack.pop();
                        let a = stack.pop();
                        stack.push(if b == 0 { 0 } else { a.wrapping_rem(b) });
                    }
                    op::LOAD_BASE..=op::LOAD_END => {
                        let bank_idx = (instruction - op::LOAD_BASE) as usize;
                        let addr = self.agent.genome[pc] as usize;
                        pc += 1;

                        let value = unsafe { (*bank_ptrs[bank_idx])[addr & 0xFF] };
                        stack.push(value);
                    }
                    op::LOAD_IND_BASE..=op::LOAD_IND_END => {
                        let bank_idx = (instruction - op::LOAD_IND_BASE) as usize;
                        let addr = stack.pop() as usize;

                        let value = unsafe { (*bank_ptrs[bank_idx])[addr & 0xFF] };
                        stack.push(value);
                    }
                    op::STORE_BASE..=op::STORE_END => {
                        let bank_idx = (instruction - op::STORE_BASE) as usize;
                        let addr = self.agent.genome[pc] as usize;
                        pc += 1;
                        let value = stack.pop();

                        unsafe { *(bank_ptrs[bank_idx] as *mut u8).add(addr & 0xFF) = value };
                    }
                    op::STORE_IND_BASE..=op::STORE_IND_END => {
                        let bank_idx = (instruction - op::STORE_IND_BASE) as usize;
                        let addr = stack.pop() as usize;
                        let value = stack.pop();

                        unsafe { *(bank_ptrs[bank_idx] as *mut u8).add(addr & 0xFF) = value };
                    }
                    op::LOADC_BASE..=op::LOADC_END => {
                        let bank_idx = (instruction - op::LOADC_BASE) as usize;
                        let addr = self.agent.genome[pc] as usize;
                        pc += 1;
                        let count = self.agent.genome[pc] as usize;
                        pc += 1;

                        for i in 0..count {
                            stack.push(unsafe { (*bank_ptrs[bank_idx])[(addr + i) & 0xFF] });
                        }
                        let log_scale = (usize::BITS - count.leading_zeros()) as f32;
                        self.agent.energy -= self.op_costs[instruction as usize] * log_scale;
                    }
                    op::LOADC_IND_BASE..=op::LOADC_IND_END => {
                        let bank_idx = (instruction - op::LOADC_IND_BASE) as usize;
                        let addr = stack.pop() as usize;
                        let count = stack.pop() as usize;

                        for i in 0..count {
                            stack.push(unsafe { (*bank_ptrs[bank_idx])[(addr + i) & 0xFF] });
                        }
                        let log_scale = (usize::BITS - count.leading_zeros()) as f32;
                        self.agent.energy -= self.op_costs[instruction as usize] * log_scale;
                    }
                    op::STOREC_BASE..=op::STOREC_END => {
                        let bank_idx = (instruction - op::STOREC_BASE) as usize;
                        let addr = self.agent.genome[pc] as usize;
                        pc += 1;
                        let count = self.agent.genome[pc] as usize;
                        pc += 1;

                        for i in 0..count {
                            let value = stack.pop();
                            unsafe {
                                *(bank_ptrs[bank_idx] as *mut u8).add((addr + i) & 0xFF) = value
                            };
                        }
                        let log_scale = (usize::BITS - count.leading_zeros()) as f32;
                        self.agent.energy -= self.op_costs[instruction as usize] * log_scale;
                    }
                    op::STOREC_IND_BASE..=op::STOREC_IND_END => {
                        let bank_idx = (instruction - op::STOREC_IND_BASE) as usize;
                        let addr = stack.pop() as usize;
                        let count = stack.pop() as usize;

                        for i in 0..count {
                            let value = stack.pop();
                            unsafe {
                                *(bank_ptrs[bank_idx] as *mut u8).add((addr + i) & 0xFF) = value
                            };
                        }
                        let log_scale = (usize::BITS - count.leading_zeros()) as f32;
                        self.agent.energy -= self.op_costs[instruction as usize] * log_scale;
                    }
                    op::JUMP => {
                        let offset = self.agent.genome[pc] as i8 as isize;
                        pc = wrap_pc(pc as isize + 1 + offset);
                    }
                    op::JUMP_IF => {
                        let offset = self.agent.genome[pc] as i8 as isize;
                        pc += 1;
                        let cond = stack.pop();
                        if cond != 0 {
                            pc = wrap_pc(pc as isize + offset);
                        }
                    }
                    op::JUMP_IF_NOT => {
                        let offset = self.agent.genome[pc] as i8 as isize;
                        pc += 1;
                        let cond = stack.pop();
                        if cond == 0 {
                            pc = wrap_pc(pc as isize + offset);
                        }
                    }
                    op::EQ => {
                        let b = stack.pop();
                        let a = stack.pop();
                        stack.push(if a == b { 1 } else { 0 });
                    }
                    op::GT => {
                        let b = stack.pop();
                        let a = stack.pop();
                        stack.push(if a > b { 1 } else { 0 });
                    }
                    op::LT => {
                        let b = stack.pop();
                        let a = stack.pop();
                        stack.push(if a < b { 1 } else { 0 });
                    }
                    op::CALL => {
                        let offset = self.agent.genome[pc] as i8 as isize;
                        pc += 1;
                        call_stack.push(pc);
                        pc = wrap_pc(pc as isize + offset);
                    }

                    op::CALL_IND => {
                        let offset = stack.pop() as i8 as isize;
                        call_stack.push(pc);
                        pc = wrap_pc(pc as isize + offset);
                    }
                    op::RET => {
                        if let Some(ret_addr) = call_stack.pop() {
                            pc = ret_addr;
                        }
                    }
                    op::REF_IND => {
                        let offset = stack.pop() as i8 as isize;
                        let value = stack.pop();
                        let target = wrap_pc(pc as isize + offset);
                        self.agent.genome[target] = value;
                    }
                    op::GET_SP => {
                        stack.push(stack.len() as u8);
                    }
                    op::GET_PC => {
                        stack.push(pc as u8);
                    }
                    op::GET_ENERGY => {
                        stack.push(self.agent.energy.0 as u8);
                    }
                    op::RNG => {
                        let mut val = (nbr_executed as usize)
                            .wrapping_add(pc)
                            .wrapping_add(stack.len())
                            .wrapping_add(self.agent.energy.0 as usize);

                        // Simple high-speed bit-mixer
                        // This ensures that even if inputs only changes by small margins,
                        // the resulting u8 is wildly different.
                        val = (val ^ (val >> 16)).wrapping_mul(0x85ebca6b);
                        val = (val ^ (val >> 13)).wrapping_mul(0xc2b2ae35);
                        val ^= val >> 16;

                        stack.push(val as u8);
                    }
                    _ => {}
                }

                break;
            }

            self.agent.energy -= self.op_costs[instruction as usize];
            nbr_executed += 1;
        }

        if pc >= self.agent.genome.len() {
            ExecutionSummary {
                reason: TerminationReason::TimedOut,
                syscalls,
            }
        } else {
            ExecutionSummary {
                reason: TerminationReason::Died,
                syscalls,
            }
        }
    }
}
