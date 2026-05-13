use std::sync::Arc;

use crate::neural::agent_memory::PrivateBanks;
use ordered_float::OrderedFloat;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Genome(pub Arc<Vec<u8>>);

#[derive(Eq, PartialEq, PartialOrd, Ord, Debug)]
pub struct Agent {
    pub(crate) base_genome: Genome,
    pub(crate) genome: Vec<u8>,
    pub(crate) private_banks: PrivateBanks,
    pub(crate) energy: OrderedFloat<f32>,
}

impl Genome {
    pub fn new(data: Vec<u8>) -> Self {
        Self(Arc::new(data))
    }
}

impl Default for Agent {
    fn default() -> Self {
        Agent {
            base_genome: Genome(Arc::new(Vec::with_capacity(32))),
            genome: Vec::with_capacity(32),
            private_banks: PrivateBanks::default(),
            energy: OrderedFloat(100.0),
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
