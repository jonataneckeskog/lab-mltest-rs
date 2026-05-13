use ordered_float::OrderedFloat;
use rand::Rng;
use std::sync::Arc;

// Assuming these imports are correct for your crate structure
use crate::neural::{Agent, agent::Genome, agent_memory::PrivateBanks};

pub struct AgentSpawner {
    pub spawn_energy: f32,
}

impl AgentSpawner {
    pub fn new_random(&self) -> Agent {
        let mut rng = rand::rng();
        let mut data = vec![0u8; 64];
        rng.fill_bytes(&mut data[..32]);

        let base_genome = Genome(Arc::new(data.clone()));

        Agent {
            base_genome,
            genome: data,
            energy: OrderedFloat(self.spawn_energy),
            private_banks: PrivateBanks::default(),
        }
    }

    pub fn spawn_standard(&self, parent_blueprint: Genome) -> Agent {
        let active_code = (*parent_blueprint.0).clone();

        Agent {
            base_genome: parent_blueprint,
            genome: active_code,
            energy: OrderedFloat(self.spawn_energy),
            private_banks: PrivateBanks::default(),
        }
    }

    pub fn spawn_from_data(blueprint_data: Vec<u8>, energy: f32) -> Agent {
        let base_genome = Genome(Arc::new(blueprint_data.clone()));

        Agent {
            base_genome,
            genome: blueprint_data,
            energy: OrderedFloat(energy),
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

        let non_zero_count = agent.genome[..32].iter().filter(|&&b| b != 0).count();
        assert!(
            non_zero_count > 0,
            "Genome should have some randomized data"
        );
        for byte in &agent.genome[32..] {
            assert_eq!(*byte, 0);
        }
        assert_eq!(&*agent.base_genome.0, &agent.genome);
    }
}
