use ordered_float::OrderedFloat;
use rand::Rng;

use crate::neural::{Agent, agent_memory::PrivateBanks};

pub struct AgentSpawner {
    pub spawn_energy: f32,
}

impl AgentSpawner {
    pub fn new_random(&self) -> Agent {
        let mut rng = rand::rng();
        let mut genome = vec![0u8; 64];
        rng.fill_bytes(&mut genome[..32]);

        Agent {
            genome,
            energy: OrderedFloat(self.spawn_energy),
            private_banks: PrivateBanks::default(),
        }
    }

    pub fn from_genome(&self, genome: Vec<u8>) -> Agent {
        Agent {
            genome,
            energy: OrderedFloat(self.spawn_energy),
            private_banks: PrivateBanks::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_agent_generation() {
        let spawner = AgentSpawner {
            spawn_energy: 100.0,
        };
        let agent = spawner.new_random();

        assert_eq!(agent.genome.len(), 64);
        assert_eq!(agent.energy.0, 100.0);
        assert_eq!(agent.private_banks, PrivateBanks::default());
        for byte in &agent.genome[..32] {
            assert_ne!(*byte, 0);
        }
        for byte in &agent.genome[32..] {
            assert_eq!(*byte, 0);
        }
    }
}
