use ordered_float::OrderedFloat;
use std::sync::Arc;

use crate::neural::{Agent, config::MutationSettings, genome::Genome};

pub struct AgentSpawner {
    pub spawn_energy: f32,
}

impl AgentSpawner {
    pub fn new_random(&self, rng: &mut impl rand::Rng) -> Agent {
        let mut data = vec![0u8; 64];
        rng.fill_bytes(&mut data[..32]);

        let base_genome = Genome(Arc::new(data.clone()));

        let mut agent = Agent::default();
        agent.base_genome = base_genome;
        agent.genome = data;
        agent.energy = OrderedFloat(self.spawn_energy);
        agent.set_mutation_settings(MutationSettings::new_random(rng));
        agent
    }

    pub fn spawn_standard(&self, parent: &Agent) -> Agent {
        let active_code = parent.genome.clone();
        let base_genome = parent.base_genome.clone();

        let mut child = Agent::default();
        child.base_genome = base_genome;
        child.genome = active_code;
        child.energy = OrderedFloat(self.spawn_energy);
        child.set_mutation_settings(parent.get_mutation_settings());
        child
    }

    pub fn spawn_from_data(blueprint_data: Vec<u8>, energy: f32) -> Agent {
        let base_genome = Genome(Arc::new(blueprint_data.clone()));

        let mut agent = Agent::default();
        agent.base_genome = base_genome;
        agent.genome = blueprint_data;
        agent.energy = OrderedFloat(energy);
        agent.set_mutation_settings(MutationSettings::default());
        agent
    }
}
