use lab_mltest::neural::AgentId;
use lab_mltest::sim::{CommunityId, Multiverse, SimulationRunner, SingleStepTask};
use lab_mltest::vm::AgentExecutor;
use lab_mltest::vm::isa::op;

struct ConstantTask {
    target: u8,
}

impl SingleStepTask for ConstantTask {
    fn input_data(&self) -> &[u8] {
        &[] // No input needed
    }

    fn evaluate(&self, output: &[u8]) -> f32 {
        // Reward agents that wrote the target value to address 0
        // (Note: output[0] corresponds to Bank 1 address 1 because Bank 1[0] is length)
        if output.get(0) == Some(&self.target) {
            1.0
        } else {
            0.0
        }
    }
}

fn main() -> anyhow::Result<()> {
    println!("--- Initializing Random Multiverse ---");

    // Create a multiverse with 2 communities, 5 agents each, 100 energy start
    let mut multiverse = Multiverse::new_random(2, 5, 10.0);

    // Pick one agent to be "Smart" manually for testing the task
    let smart_id = AgentId(0);
    let comm_id = CommunityId(0);

    {
        let sess = multiverse.session(comm_id, smart_id)?;
        sess.agent.set_genome(vec![
            op::PUSH,
            1, // length
            op::PUSH,
            0,                      // address 0
            op::STORE_IND_BASE + 1, // Store to Bank 1
            op::PUSH,
            42, // value
            op::PUSH,
            1,                      // address 1
            op::STORE_IND_BASE + 1, // Store to Bank 1
            op::HALT,
        ]);
    }

    let executor = AgentExecutor::new(&[0.1; 256]);
    let runner = SimulationRunner::new(&executor);
    let task = ConstantTask { target: 42 };

    println!("\n--- Starting Simulation Loop ---");

    for generation in 0..21 {
        // Run one tick with 1000 total energy budget
        runner.run_population_tick(&mut multiverse, &task, 1000.0, 100);

        let survivor_count = multiverse.survivor_count();

        let smart_sess = multiverse.session(comm_id, smart_id);
        if let Ok(sess) = smart_sess {
            let energy = sess.agent.get_energy();
            let output = sess.agent.collect_output();
            println!("Gen {:02}: Smart Agent Energy: {:.2}, Output: {:?}, Survivors: {}", 
                     generation, energy, output, survivor_count);
        } else {
            println!("Gen {:02}: Smart Agent DIED, Survivors: {}", generation, survivor_count);
        }


        if generation % 10 == 0 {
            let save_path = format!("checkpoints/gen_{}", generation);
            println!(">> Saving checkpoint to {}", save_path);
            multiverse.save_to(&save_path)?;
        }
    }

    println!("\n--- Simulation Complete ---");
    Ok(())
}
