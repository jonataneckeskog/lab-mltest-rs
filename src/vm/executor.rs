use crate::vm::{
    isa::op,
    stack::ByteStack,
    traits::{VmContext, VmMemory},
};

pub struct AgentExecutor<'a> {
    op_costs: &'a [f32; 256],
}

pub struct ExecutionSummary {
    pub reason: TerminationReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminationReason {
    Halted { confidence: u8 },
    TimedOut,
    Died,
}

impl<'a> AgentExecutor<'a> {
    pub fn new(op_costs: &'a [f32; 256]) -> Self {
        Self { op_costs }
    }

    pub fn run<M: VmMemory, C: VmContext>(
        &self,
        memory: &mut M,
        ctx: &mut C,
        max_steps: usize,
    ) -> ExecutionSummary {
        let mut pc = 0;
        let mut nbr_executed = 0;
        let mut stack = ByteStack::new();
        let mut call_stack: Vec<usize> = Vec::with_capacity(8);

        let genome_mask = memory.genome_len().saturating_sub(1);
        let wrap_pc = |pos: isize| -> usize { (pos as usize) & genome_mask };
        macro_rules! syst_call {
            ($ctx:expr, $stack:expr, $instr:expr) => {
                if !$ctx.syscall($instr, &mut $stack) {
                    return ExecutionSummary {
                        reason: TerminationReason::Halted { confidence: 0 },
                    };
                }
            };
        }

        while nbr_executed < max_steps && pc < memory.genome_len() && memory.get_energy() > 0.0 {
            let mut instruction = memory.read_genome(pc);
            pc += 1;

            loop {
                match instruction {
                    op::NO_OP => {}
                    op::HALT => {
                        let confidence = stack.pop();
                        return ExecutionSummary {
                            reason: TerminationReason::Halted { confidence },
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
                        let value = memory.read_genome(pc);
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
                        memory.consume_energy(self.op_costs[op::EXEC_STACK as usize]);
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
                        let addr = memory.read_genome(pc);
                        pc += 1;
                        stack.push(memory.read_bank(bank_idx, addr));
                    }
                    op::LOAD_IND_BASE..=op::LOAD_IND_END => {
                        let bank_idx = (instruction - op::LOAD_IND_BASE) as usize;
                        let addr = stack.pop();
                        stack.push(memory.read_bank(bank_idx, addr));
                    }
                    op::STORE_BASE..=op::STORE_END => {
                        let bank_idx = (instruction - op::STORE_BASE) as usize;
                        let addr = memory.read_genome(pc);
                        pc += 1;
                        let value = stack.pop();
                        memory.write_bank(bank_idx, addr, value);
                    }
                    op::STORE_IND_BASE..=op::STORE_IND_END => {
                        let bank_idx = (instruction - op::STORE_IND_BASE) as usize;
                        let addr = stack.pop();
                        let value = stack.pop();
                        memory.write_bank(bank_idx, addr, value);
                    }
                    op::LOADC_BASE..=op::LOADC_END => {
                        let bank_idx = (instruction - op::LOADC_BASE) as usize;
                        let addr = memory.read_genome(pc);
                        pc += 1;
                        let count = memory.read_genome(pc) as usize;
                        pc += 1;

                        for i in 0..count {
                            stack.push(memory.read_bank(bank_idx, addr.wrapping_add(i as u8)));
                        }
                        let log_scale = (usize::BITS - count.leading_zeros()) as f32;
                        memory.consume_energy(self.op_costs[instruction as usize] * log_scale);
                    }
                    op::LOADC_IND_BASE..=op::LOADC_IND_END => {
                        let bank_idx = (instruction - op::LOADC_IND_BASE) as usize;
                        let addr = stack.pop();
                        let count = stack.pop() as usize;

                        for i in 0..count {
                            stack.push(memory.read_bank(bank_idx, addr.wrapping_add(i as u8)));
                        }
                        let log_scale = (usize::BITS - count.leading_zeros()) as f32;
                        memory.consume_energy(self.op_costs[instruction as usize] * log_scale);
                    }
                    op::STOREC_BASE..=op::STOREC_END => {
                        let bank_idx = (instruction - op::STOREC_BASE) as usize;
                        let addr = memory.read_genome(pc);
                        pc += 1;
                        let count = memory.read_genome(pc) as usize;
                        pc += 1;

                        for i in 0..count {
                            let value = stack.pop();
                            memory.write_bank(bank_idx, addr.wrapping_add(i as u8), value);
                        }
                        let log_scale = (usize::BITS - count.leading_zeros()) as f32;
                        memory.consume_energy(self.op_costs[instruction as usize] * log_scale);
                    }
                    op::STOREC_IND_BASE..=op::STOREC_IND_END => {
                        let bank_idx = (instruction - op::STOREC_IND_BASE) as usize;
                        let addr = stack.pop();
                        let count = stack.pop() as usize;

                        for i in 0..count {
                            let value = stack.pop();
                            memory.write_bank(bank_idx, addr.wrapping_add(i as u8), value);
                        }
                        let log_scale = (usize::BITS - count.leading_zeros()) as f32;
                        memory.consume_energy(self.op_costs[instruction as usize] * log_scale);
                    }
                    op::JUMP => {
                        let offset = memory.read_genome(pc) as i8 as isize;
                        pc = wrap_pc(pc as isize + 1 + offset);
                    }
                    op::JUMP_IF => {
                        let offset = memory.read_genome(pc) as i8 as isize;
                        pc += 1;
                        let cond = stack.pop();
                        if cond != 0 {
                            pc = wrap_pc(pc as isize + offset);
                        }
                    }
                    op::JUMP_IF_NOT => {
                        let offset = memory.read_genome(pc) as i8 as isize;
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
                        let offset = memory.read_genome(pc) as i8 as isize;
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
                        memory.write_genome(target, value);
                    }
                    op::DOUBLE_SIZE => {
                        let new_size = memory.genome_len() * 2;
                        if new_size <= 4096 {
                            memory.resize_genome(new_size);
                        }
                    }
                    op::HALF_SIZE => {
                        let new_size = memory.genome_len() / 2;
                        if new_size >= 32 {
                            memory.resize_genome(new_size);
                        }
                    }
                    op::DIE => {
                        return ExecutionSummary {
                            reason: TerminationReason::Died,
                        };
                    }
                    op::LEAVE_COMMUNITY => {
                        syst_call!(ctx, stack, instruction);
                    }
                    op::SPAWN_CHILD => {
                        let spent_energy = stack.peek() as f32 * 0.39215686;
                        if spent_energy <= memory.get_energy() {
                            syst_call!(ctx, stack, instruction);
                        }
                    }
                    op::GET_SP => {
                        stack.push(stack.len() as u8);
                    }
                    op::GET_PC => {
                        stack.push(pc as u8);
                    }
                    op::GET_ENERGY => {
                        stack.push(memory.get_energy() as u8);
                    }
                    op::GET_ID => {
                        stack.push(ctx.agent_id());
                    }
                    op::GET_COMMUNITY_ID => {
                        stack.push(ctx.community_id());
                    }
                    op::RNG => {
                        let val = (nbr_executed as usize)
                            .wrapping_add(pc)
                            .wrapping_add(stack.len())
                            .wrapping_add(memory.get_energy() as usize);

                        stack.push(ctx.random_byte(val));
                    }
                    _ => {}
                }

                break;
            }

            memory.consume_energy(self.op_costs[instruction as usize]);
            nbr_executed += 1;
        }

        if pc >= memory.genome_len() {
            ExecutionSummary {
                reason: TerminationReason::TimedOut,
            }
        } else {
            ExecutionSummary {
                reason: TerminationReason::Died,
            }
        }
    }
}
