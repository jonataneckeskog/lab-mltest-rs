use crate::core::SingleStepTask;
use crate::evolution::hooks::{CheckpointHook, PopulationTargetHook, PrintStatsHook};
use crate::evolution::{EvolutionConfig, EvolutionEngine, EvolutionHook};
use crate::neural::AgentSpawner;
use crate::sim::Multiverse;
use std::path::PathBuf;

/// Creates a standard simulation setup for testing tasks.
pub fn create_test_sim<'a>(
    communities: usize,
    pop_per_comm: usize,
    task: &'a dyn SingleStepTask,
) -> (EvolutionEngine<'a>, Vec<Box<dyn EvolutionHook>>) {
    let config = EvolutionConfig {
        communities,
        min_population: pop_per_comm,
        starting_energy: 5.0,
        tick_energy_budget: 20.0,
        ticks_per_gen: 150,
        max_generations: 1000,
    };

    let (engine, mut hooks) = create_pluggable_engine(config, task, Vec::new());

    hooks.push(Box::new(PrintStatsHook {
        interval: 10,
        highest_survivors: 0,
    }));
    hooks.push(Box::new(PopulationTargetHook {
        target: communities * pop_per_comm,
    }));
    hooks.push(Box::new(CheckpointHook {
        interval: 500,
        dir: PathBuf::from("checkpoints"),
    }));

    (engine, hooks)
}

/// Creates an engine with a specific config and a set of initial hooks.
pub fn create_pluggable_engine<'a>(
    config: EvolutionConfig,
    task: &'a dyn SingleStepTask,
    hooks: Vec<Box<dyn EvolutionHook>>,
) -> (EvolutionEngine<'a>, Vec<Box<dyn EvolutionHook>>) {
    let rng = &mut rand::rng();
    let multiverse = Multiverse::new_random(
        rng,
        config.communities,
        config.min_population,
        config.starting_energy,
    );

    let spawner = AgentSpawner {
        spawn_energy: config.starting_energy,
    };

    let engine = EvolutionEngine::new(config, multiverse, task, spawner);

    (engine, hooks)
}
