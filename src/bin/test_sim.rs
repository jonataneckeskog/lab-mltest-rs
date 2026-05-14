use lab_mltest::neural::AgentId;
use lab_mltest::sim::{CommunityId, Multiverse, SimulationRunner, SingleStepTask};
use lab_mltest::vm::AgentExecutor;
use lab_mltest::vm::isa::op;

struct SimConfig {
    communities: usize,
    agents_per_comm: usize,
    starting_energy: f32,
    tick_energy_budget: f32,
    ticks_per_gen: usize,
    max_generations: usize,
    target_value: u8,
    save_interval: usize,
}

struct ConstantTask {
    target: u8,
}

impl SingleStepTask for ConstantTask {
    fn input_data(&self) -> &[u8] {
        &[]
    }

    fn evaluate(&self, output: &[u8]) -> f32 {
        if output.get(0) == Some(&self.target) {
            1.0
        } else {
            0.0
        }
    }
}

fn main() -> anyhow::Result<()> {
    let config = SimConfig {
        communities: 2,
        agents_per_comm: 50, // Bumped up for real evolution
        starting_energy: 10.0,
        tick_energy_budget: 1000.0,
        ticks_per_gen: 100,
        max_generations: 500,
        target_value: 42,
        save_interval: 50,
    };

    println!("--- Initializing Random Multiverse ---");
    let rng = &mut rand::rng();
    let mut multiverse = Multiverse::new_random(
        rng,
        config.communities,
        config.agents_per_comm,
        config.starting_energy,
    );

    // Optional: Still inject a seed agent if you want to test propagation
    inject_seed_agent(&mut multiverse, config.target_value)?;

    let executor = AgentExecutor::new(&[0.1; 256]);
    let runner = SimulationRunner::new(&executor);
    let task = ConstantTask {
        target: config.target_value,
    };

    println!("\n--- Starting Simulation Loop ---");

    let mut highest_survivors = 0;

    for generation in 0..=config.max_generations {
        runner.run_population_tick(
            rng,
            &mut multiverse,
            &task,
            config.tick_energy_budget,
            config.ticks_per_gen,
        );

        let survivor_count = multiverse.survivor_count();

        // Check for extinction
        if survivor_count == 0 {
            println!(
                "Gen {:03}: ☠️ Population went extinct. Halting simulation.",
                generation
            );
            break;
        }

        // Ideally, query your multiverse for the best agent's stats here
        // let best_agent_energy = multiverse.get_max_energy();

        println!(
            "Gen {:03} | Survivors: {} | ... (add avg/max fitness here)",
            generation, survivor_count
        );

        // Smart Checkpointing: Save periodically OR if we hit a new milestone
        if generation > 0 && generation % config.save_interval == 0 {
            let path = format!("checkpoints/gen_{}", generation);
            multiverse.save_to(&path)?;
        }

        if survivor_count > highest_survivors {
            highest_survivors = survivor_count;
            println!(">> 🏆 New survivor record! Saving best_model...");
            multiverse.save_to("checkpoints/best_model")?;
        }

        // Early stopping condition (e.g., 90% of max possible population survived)
        let total_capacity = config.communities * config.agents_per_comm;
        if survivor_count >= (total_capacity as f32 * 0.9) as usize {
            println!("🎯 Convergence reached! 90% survival rate.");
            break;
        }
    }

    println!("\n--- Simulation Complete ---");
    Ok(())
}

fn inject_seed_agent(multiverse: &mut Multiverse, target: u8) -> anyhow::Result<()> {
    let smart_id = AgentId(0);
    let comm_id = CommunityId(0);
    if let Ok(sess) = multiverse.session(comm_id, smart_id) {
        sess.agent.set_genome(vec![
            op::PUSH,
            1,
            op::PUSH,
            0,
            op::STORE_IND_BASE + 1,
            op::PUSH,
            target,
            op::PUSH,
            1,
            op::STORE_IND_BASE + 1,
            op::HALT,
        ]);
    }
    Ok(())
}
