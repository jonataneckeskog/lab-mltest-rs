use lab_mltest::evolution::{
    BestModelHook, CheckpointHook, EvolutionConfig, EvolutionEngine, EvolutionHook,
    PopulationTargetHook, PrintStatsHook,
};
use lab_mltest::neural::AgentSpawner;
use lab_mltest::sim::{Multiverse, SingleStepTask};
use std::path::PathBuf;

struct ConstantTask {
    target: u8,
}

impl SingleStepTask for ConstantTask {
    fn input_data(&self) -> &[u8] {
        &[]
    }
    fn evaluate(&self, output: &[u8]) -> f32 {
        if output.is_empty() {
            return 0.0;
        }
        let mut score = 0.1;
        let diff = (output[0] as i16 - self.target as i16).abs();
        score += (1.0 - (diff as f32 / 255.0)) * 0.4;
        if output[0] == self.target {
            score += 0.5;
        }
        score
    }
}

fn main() -> anyhow::Result<()> {
    let config = EvolutionConfig {
        communities: 4,
        min_population: 20,
        starting_energy: 5.0,
        tick_energy_budget: 20.0,
        ticks_per_gen: 150,
        max_generations: 1000,
    };

    let rng = &mut rand::rng();
    let multiverse = Multiverse::new_random(
        rng,
        config.communities,
        config.min_population,
        config.starting_energy,
    );

    let task = ConstantTask { target: 42 };
    let spawner = AgentSpawner {
        spawn_energy: config.starting_energy,
    };

    let mut engine = EvolutionEngine::new(config, multiverse, &task, spawner);

    let mut hooks: Vec<Box<dyn EvolutionHook>> = Vec::new();
    hooks.push(Box::new(PrintStatsHook {
        interval: 10,
        highest_survivors: 0,
    }));
    hooks.push(Box::new(PopulationTargetHook { target: 4 * 20 }));
    hooks.push(Box::new(CheckpointHook {
        interval: 500,
        dir: PathBuf::from("checkpoints"),
    }));
    hooks.push(Box::new(BestModelHook {
        highest_survivors: 0,
        path: "checkpoints/best_engine_model".to_string(),
    }));

    println!("--- Starting Evolution Engine ---");
    engine.run(rng, hooks)?;
    println!("--- Engine Run Complete ---");

    Ok(())
}
