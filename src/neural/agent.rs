use super::byte_stack::ByteStack;

use crate::neural::{
    agent_memory::{PrivateBanks, SharedBanks},
    opcode::op,
};
use ordered_float::OrderedFloat;

#[derive(Eq, PartialEq, PartialOrd, Ord, Debug)]
pub struct Agent {
    pub(crate) genome: Vec<u8>,
    pub(crate) energy: OrderedFloat<f32>,
    pub(crate) private_banks: PrivateBanks,
}

pub enum ExecutionResult {
    Halted { confidence: u8 },
    TimedOut,
    Died,
}

impl Default for Agent {
    fn default() -> Self {
        Agent {
            genome: Vec::with_capacity(32),
            energy: OrderedFloat(100.0),
            private_banks: PrivateBanks::default(),
        }
    }
}

impl Agent {
    pub fn load_input(&mut self, data: &[u8]) {
        self.private_banks.write_input(data);
    }

    pub fn execute(
        &mut self,
        _shared: &mut SharedBanks,
        max_steps: usize,
        op_costs: &[f32; 256],
    ) -> ExecutionResult {
        let mut pc = 0;
        let mut nbr_executed = 0;

        let mut stack = ByteStack::new();

        let bank_ptrs: [*const [u8; 256]; 8] = [
            &self.private_banks.0[0],
            &self.private_banks.0[1],
            &self.private_banks.0[2],
            &self.private_banks.0[3],
            &self.private_banks.0[4],
            &self.private_banks.0[5],
            &_shared.0[0],
            &_shared.0[1],
        ];

        while nbr_executed < max_steps && pc < self.genome.len() && self.energy.0 > 0.0 {
            let instruction = self.genome[pc];
            pc += 1;

            match instruction {
                op::NO_OP => {}
                op::HALT => {
                    let confidence = stack.pop();
                    return ExecutionResult::Halted { confidence };
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
                    let value = self.genome[pc];
                    stack.push(value);
                    pc += 1;
                }
                op::OVER => {
                    stack.over();
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
                    let addr = self.genome[pc] as usize;
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
                    let addr = self.genome[pc] as usize;
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
                    let addr = self.genome[pc] as usize;
                    pc += 1;
                    let count = self.genome[pc] as usize;
                    pc += 1;

                    for i in 0..count {
                        stack.push(unsafe { (*bank_ptrs[bank_idx])[(addr + i) & 0xFF] });
                    }
                }
                op::LOADC_IND_BASE..=op::LOADC_IND_END => {
                    let bank_idx = (instruction - op::LOADC_IND_BASE) as usize;
                    let addr = stack.pop() as usize;
                    let count = stack.pop() as usize;

                    for i in 0..count {
                        stack.push(unsafe { (*bank_ptrs[bank_idx])[(addr + i) & 0xFF] });
                    }
                }
                op::STOREC_BASE..=op::STOREC_END => {
                    let bank_idx = (instruction - op::STOREC_BASE) as usize;
                    let addr = self.genome[pc] as usize;
                    pc += 1;
                    let count = self.genome[pc] as usize;
                    pc += 1;

                    for i in 0..count {
                        let value = stack.pop();
                        unsafe { *(bank_ptrs[bank_idx] as *mut u8).add((addr + i) & 0xFF) = value };
                    }
                }
                op::STOREC_IND_BASE..=op::STOREC_IND_END => {
                    let bank_idx = (instruction - op::STOREC_IND_BASE) as usize;
                    let addr = stack.pop() as usize;
                    let count = stack.pop() as usize;

                    for i in 0..count {
                        let value = stack.pop();
                        unsafe { *(bank_ptrs[bank_idx] as *mut u8).add((addr + i) & 0xFF) = value };
                    }
                }
                op::JUMP => {
                    let offset = self.genome[pc] as i8;
                    pc = (pc as i32 + 1 + offset as i32).max(0) as usize;
                }
                op::JUMP_IF => {
                    let offset = self.genome[pc] as i8;
                    pc += 1;
                    let cond = stack.pop();
                    if cond != 0 {
                        pc = (pc as i32 + offset as i32).max(0) as usize;
                    }
                }
                op::JUMP_IF_NOT => {
                    let offset = self.genome[pc] as i8;
                    pc += 1;
                    let cond = stack.pop();
                    if cond == 0 {
                        pc = (pc as i32 + offset as i32).max(0) as usize;
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
                    // TODO: Call immediate address
                }
                op::CALL_IND => {
                    // TODO: Pop stack, call that address
                }
                op::RET => {
                    // TODO: Pop return stack and jump back
                }
                op::REF_IND => {
                    // TODO: Allows AI to rewrite itself
                }
                op::SELECT => {
                    let cond = stack.pop();
                    let val_b = stack.pop();
                    let val_a = stack.pop();
                    stack.push(if cond != 0 { val_a } else { val_b });
                }
                op::GET_SP => {
                    stack.push(stack.len() as u8);
                }
                op::GET_PC => {
                    stack.push(pc as u8);
                }
                op::GET_ENERGY => {
                    stack.push(self.energy.0 as u8);
                }
                op::RNG => {
                    let mut val = (nbr_executed as usize)
                        .wrapping_add(pc)
                        .wrapping_add(stack.len())
                        .wrapping_add(self.energy.0 as usize);

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

            self.energy -= op_costs[instruction as usize];
            nbr_executed += 1;
        }

        if pc >= self.genome.len() {
            ExecutionResult::TimedOut
        } else {
            ExecutionResult::Died
        }
    }

    pub fn collect_output(&self) -> Vec<u8> {
        self.private_banks.read_output()
    }
}
