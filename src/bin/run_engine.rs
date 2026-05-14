use lab_mltest::evolution::{EvolutionConfig, EvolutionEngine, EvolutionHook};
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

struct StatsHook {
    highest_survivors: usize,
    target_pop: usize,
}

impl EvolutionHook for StatsHook {
    fn on_generation_complete(&mut self, generation: usize, multiverse: &Multiverse) -> bool {
        let survivor_count = multiverse.survivor_count();
        let (_min_e, max_e, avg_e) = multiverse.get_energy_stats();

        if generation % 10 == 0 || survivor_count > self.highest_survivors {
            println!(
                "Gen {:03} | Survivors: {:4} | Max Energy: {:.2} | Avg: {:.2}",
                generation, survivor_count, max_e, avg_e
            );
        }

        if survivor_count > self.highest_survivors {
            self.highest_survivors = survivor_count;
            println!(">> 🏆 New survivor record!");
            let _ = multiverse.save_to("checkpoints/best_engine_model");
        }

        // Early exit condition
        if survivor_count >= self.target_pop {
            println!("🎯 Convergence reached!");
            return false; // Stop simulation
        }

        true // Continue simulation
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
        save_interval: Some(500),
        checkpoint_dir: Some(PathBuf::from("checkpoints")),
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
    let mut hook = StatsHook {
        highest_survivors: 0,
        target_pop: 4 * 20,
    };

    println!("--- Starting Evolution Engine ---");
    engine.run(rng, &mut hook)?;
    println!("--- Engine Run Complete ---");

    Ok(())
}
