use crate::neural::agent_memory::PrivateBanks;
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
    pub fn load_input(&mut self, data: &[u8]) {
        self.private_banks.write_input(data);
    }

    pub fn collect_output(&self) -> Vec<u8> {
        self.private_banks.read_output()
    }
}
