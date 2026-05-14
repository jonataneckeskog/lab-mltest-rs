use crate::vm::stack::ByteStack;

pub trait VmMemory {
    fn read_genome(&self, pc: usize) -> u8;
    fn write_genome(&mut self, pc: usize, val: u8);
    fn genome_len(&self) -> usize;
    fn resize_genome(&mut self, new_size: usize);

    /// Read from one of the memory banks (0-7)
    fn read_bank(&self, bank_idx: usize, addr: u8) -> u8;
    /// Write to one of the memory banks (0-7)
    fn write_bank(&mut self, bank_idx: usize, addr: u8, val: u8);

    fn get_energy(&self) -> f32;

    fn get_age(&self) -> u64;

    fn consume_energy(&mut self, amount: f32);
    }


pub trait VmContext {
    fn agent_id(&self) -> u8;
    fn community_id(&self) -> u8;
    fn random_byte(&self, seed: usize) -> u8;

    /// Handle a syscall. Returns true if execution should continue.
    fn syscall(&mut self, opcode: u8, stack: &mut ByteStack) -> bool;
}
