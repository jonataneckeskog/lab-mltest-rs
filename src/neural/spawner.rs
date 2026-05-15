use crate::neural::{
    Agent, MutationSettings,
    genome::GeneticBlueprint,
    memory::PrivateBanks,
};
use ordered_float::OrderedFloat;
use rand::RngExt;
use std::sync::Arc;

pub struct AgentSpawner {
    pub spawn_energy: f32,
}

impl AgentSpawner {
    pub fn new_random(&self, rng: &mut impl rand::Rng) -> Agent {
        let mut data = vec![0u8; 64];
        rng.fill_bytes(&mut data[..32]);

        let blueprint = Arc::new(GeneticBlueprint {
            original_sequence: data,
            mutation_settings: MutationSettings {
                cosmic_ray_rate: rng.random(),
            },
        });
        
        let mut agent = Agent {
            blueprint,
            genome: Vec::new(),
            private_banks: PrivateBanks::default(),
            energy: OrderedFloat(0.0),
            age: 0,
        };
        agent.reset(self.spawn_energy);
        agent
    }

    pub fn spawn_clone(&self, parent: &Agent) -> Agent {
        let mut agent = Agent {
            blueprint: Arc::clone(&parent.blueprint),
            genome: Vec::new(),
            private_banks: PrivateBanks::default(),
            energy: OrderedFloat(0.0),
            age: 0,
        };
        agent.reset(self.spawn_energy);
        agent
    }

    pub fn spawn_child(parent: &Agent, energy: f32) -> Agent {
        let mut agent = Agent {
            blueprint: Arc::clone(&parent.blueprint),
            genome: Vec::new(),
            private_banks: PrivateBanks::default(),
            energy: OrderedFloat(0.0),
            age: 0,
        };
        agent.reset(energy);
        agent
    }
}
