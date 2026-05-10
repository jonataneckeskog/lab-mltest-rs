use crate::neural::agent_memory::{PrivateBanks, SharedBanks};

pub struct Agent {
    genome: Vec<u8>,
    pc: usize,
    acc: u8,
    energy: f32,
    private_banks: PrivateBanks,
}

impl Agent {
    pub fn load_input(&mut self, data: &[u8]) -> &mut Self {
        self.private_banks.write_input(data);
        self
    }

    pub fn execute(&mut self, shared: &mut SharedBanks) -> &mut Self {
        // Logic here...
        self
    }

    pub fn collect_output(&self) -> Vec<u8> {
        self.private_banks.read_output()
    }
}
