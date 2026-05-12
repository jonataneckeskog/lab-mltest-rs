use super::byte_stack::ByteStack;
use std::default;

use crate::neural::{
    agent_memory::{PrivateBanks, SharedBanks},
    opcode::op,
};

pub struct Agent {
    genome: Vec<u8>,
    energy: f32,
    private_banks: PrivateBanks,
}

impl Agent {
    pub fn load_input(&mut self, data: &[u8]) -> &mut Self {
        self.private_banks.write_input(data);
        self
    }

    pub fn execute(&mut self, shared: &mut SharedBanks, max_steps: usize) -> &mut Self {
        let mut pc = 0;
        let mut nbr_executed = 0;

        let mut stack = ByteStack::new();
        let mut sp = 0;

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
                    let a = stack.pop();
                    let b = stack.pop();
                    stack.push(a);
                    stack.push(b);
                }
                op::PUSH => {
                    let value = self.genome[pc];
                    stack.push(value);
                    pc += 1;
                }
                _ => {}
            }

            nbr_executed += 1;
        }

        self
    }

    pub fn collect_output(&self) -> Vec<u8> {
        self.private_banks.read_output()
    }
}
