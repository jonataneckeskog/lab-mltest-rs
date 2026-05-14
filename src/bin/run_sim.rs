use lab_mltest::evolution::hooks::basics::{CheckpointHook, PrintStatsHook};
use lab_mltest::evolution::{EvolutionConfig, EvolutionHook};
use lab_mltest::tasks::constant::ConstantTask;
use lab_mltest::templates::basic::create_pluggable_engine;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let task = ConstantTask { target: 123 };

    // 1. Define a custom configuration
    let config = EvolutionConfig {
        communities: 2,
        min_population: 50,
        starting_energy: 10.0,
        tick_energy_budget: 50.0,
        ticks_per_gen: 200,
        max_generations: 500,
    };

    // 2. Prepare your own specific set of plugins/hooks
    let mut my_plugins: Vec<Box<dyn EvolutionHook>> = Vec::new();
    my_plugins.push(Box::new(PrintStatsHook {
        interval: 25, // Log less frequently
        highest_survivors: 0,
    }));
    my_plugins.push(Box::new(CheckpointHook {
        interval: 100,
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
