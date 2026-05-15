pub mod agent;
pub mod genome;
pub mod memory;
mod mutation;
pub mod spawner;

pub use agent::{Agent, AgentVmMemory};
pub use genome::GeneticBlueprint;
pub use memory::SharedBanks;
pub use mutation::MutationSettings;
pub use spawner::AgentSpawner;
