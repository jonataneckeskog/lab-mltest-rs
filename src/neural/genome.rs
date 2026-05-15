use serde::{Deserialize, Serialize};
use crate::neural::MutationSettings;

/// Represents the immutable genetic lineage of an agent.
/// This is intended to be wrapped in an Arc by the Agent.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
pub struct GeneticBlueprint {
    pub original_sequence: Vec<u8>,
    pub mutation_settings: MutationSettings,
}

impl GeneticBlueprint {
    pub fn new(data: Vec<u8>, mutation_settings: MutationSettings) -> Self {
        Self {
            original_sequence: data,
            mutation_settings,
        }
    }
}

impl Default for GeneticBlueprint {
    fn default() -> Self {
        Self {
            original_sequence: Vec::with_capacity(32),
            mutation_settings: MutationSettings { cosmic_ray_rate: 0 },
        }
    }
}
