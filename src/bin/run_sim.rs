use lab_mltest::evolution::hooks::basics::{
    CheckpointHook, PopulationBalancerHook, PrintStatsHook,
};
use lab_mltest::evolution::{EvolutionConfig, EvolutionHook, create_pluggable_engine};
use lab_mltest::tasks::constant::ConstantTask;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let task = ConstantTask { target: 123 };

    // 1. Define a custom configuration
    let config = EvolutionConfig {
        communities: 2,
        min_population: 2,
        starting_energy: 10.0,
        tick_energy_budget: 10.0,
        ticks_per_gen: 100,
        max_generations: 1000,
    };

    // 2. Prepare your own specific set of plugins/hooks
    let mut my_plugins: Vec<Box<dyn EvolutionHook>> = Vec::new();
    my_plugins.push(Box::new(PopulationBalancerHook {
        min_population: 5,
        refill_fn: Box::new(move || {
            (lab_mltest::neural::AgentSpawner {
                spawn_energy: config.starting_energy,
            })
            .new_random(&mut rand::rng())
        }),
        rng: Box::new(rand::rng()),
    }));
    my_plugins.push(Box::new(PrintStatsHook {
        interval: 25, // Log less frequently
        highest_survivors: 0,
    }));
    my_plugins.push(Box::new(CheckpointHook {
        interval: 50,
        dir: PathBuf::from("sim_checkpoints"),
    }));

    // 3. Use the pluggable template to wire everything together
    // This handles the Multiverse and Spawner creation for you.
    let (mut engine, hooks) = create_pluggable_engine(config, &task, my_plugins);

    println!("--- Starting Pluggable Simulation ---");
    let rng = &mut rand::rng();
    engine.run(rng, hooks)?;
    println!("--- Simulation Complete ---");

    Ok(())
}
