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
    pub fn load_input(&mut self, data: &[u8]) -> &mut Self {
        self.private_banks.write_input(data);
        self
    }

    pub fn execute(
        &mut self,
        _shared: &mut SharedBanks,
        max_steps: usize,
        op_costs: &[f32; 256],
    ) -> &mut Self {
        let mut pc = 0;
        let mut nbr_executed = 0;

        let mut stack = ByteStack::new();

        while nbr_executed < max_steps && pc < self.genome.len() {
            let instruction = self.genome[pc];
            pc += 1;

            match instruction {
                op::NO_OP => {}
                op::HALT => break,
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
                op::LOAD_BASE | op::LOAD_IND_BASE => {
                    // TODO: Memory Interaction (Load)
                }
                op::STORE_BASE | op::STORE_IND_BASE => {
                    // TODO: Memory Interaction (Store)
                }
                op::LOADC_BASE | op::LOADC_IND_BASE => {
                    // TODO: Memory Interaction (Copy chunks)
                    // Note: STOREC_BASE and STOREC_IND_BASE share the same opcodes
                }
                op::JUMP => {
                    // TODO: Control flow jump
                }
                op::JUMP_IF => {
                    // TODO: Control flow jump if
                }
                op::JUMP_IF_NOT => {
                    // TODO: Control flow jump if not
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
                    // TODO: Pushes the Stack Pointer to stack
                }
                op::GET_PC => {
                    // TODO: Pushes the Program Counter to stack
                }
                op::GET_ENERGY => {
                    // TODO: Pushes the current Energy to stack
                }
                op::RNG => {
                    // TODO: Pushes a random byte
                }
                _ => {}
            }

            self.energy -= op_costs[instruction as usize];
            nbr_executed += 1;
        }

        self
    }

    pub fn collect_output(&self) -> Vec<u8> {
        self.private_banks.read_output()
    }
}
