use ordered_float::OrderedFloat;
use rand::Rng;
use std::sync::Arc;

use crate::neural::{Agent, genome::Genome, memory::PrivateBanks};

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
