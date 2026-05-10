use crate::neural::memory::{PrivateBanks, SharedBanks};

pub struct Agent {
    genome: Vec<u8>,
    pc: usize,
    acc: u8,
    energy: f32,
    private_banks: PrivateBanks,
}

pub struct Community {
    agents: Vec<Agent>,
    shared_comms: SharedBanks,
}

impl Agent {
    pub fn execute(&mut self, shared: &mut SharedBanks) {
        // Logic here...
        let val = shared[0][10]; // Read from b6
        shared[1][20] = self.acc; // Write to b7
    }
}
