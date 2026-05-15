use crate::{
    core::{AgentId, SingleStepTask},
    neural::{Agent, AgentVmMemory},
    sim::core::{SimulationContext, SimulationEvent},
    sim::state::Multiverse,
    sim::runner::engine::SimulationRunner,
    vm::TerminationReason,
};

impl<'a> SimulationRunner<'a> {
    /// Orchestrate a full population tick: run agents, distribute energy, and resolve events.
    pub fn run_population_tick(
        &self,
        multiverse: &mut Multiverse,
        task: &dyn SingleStepTask,
        total_energy: f32,
        max_steps: usize,
        min_population: usize,
        mut refill_fn: impl FnMut() -> Agent,
    ) {
        let (scores, all_events, dead_agents) =
            self.run_population(multiverse, task, max_steps);

        // 1. Kill agents that died during execution
        for agent_id in dead_agents {
            multiverse.remove_agent(agent_id);
        }

        // 2. Distribute Energy (Additive rewards)
        self.distribute_energy_by_scores(multiverse, scores, total_energy);

        // 3. Resolve Events (Spawn/Migration)
        crate::sim::core::resolve_events(multiverse, all_events);

        // 4. Population Balancing
        while multiverse.population < min_population {
            let new_agent = refill_fn();
            multiverse.add_agent_to_random_community(new_agent);
        }
    }

    /// Run a single step task across an entire population.
    /// Returns scores, triggered events, and a list of agents that explicitly died.
    pub fn run_population(
        &self,
        multiverse: &mut Multiverse,
        task: &dyn SingleStepTask,
        max_steps: usize,
    ) -> (
        Vec<(AgentId, f32)>,
        Vec<SimulationEvent>,
        Vec<AgentId>,
    ) {
        let mut all_events = Vec::new();
        let mut scores = Vec::new();
        let mut dead_agents = Vec::new();
        let input = task.input_data();

        // Use the flattened iterator to run all agents
        for (comm_id, community) in &mut multiverse.spaces {
            for (agent_id, agent) in &mut community.agents {
                agent.age += 1;
                agent.load_input(input);

                let mut ctx = SimulationContext::new(*agent_id, *comm_id);
                let summary = {
                    let mut memory = AgentVmMemory::new(agent, &mut community.shared_comms);
                    self.executor.run(&mut memory, &mut ctx, max_steps)
                };

                match summary.reason {
                    TerminationReason::Died => {
                        dead_agents.push(*agent_id);
                        continue; // Skip scoring and event processing for this agent
                    }
                    _ => {
                        let output = agent.collect_output();
                        let score = task.evaluate(&output).max(0.0);

                        scores.push((*agent_id, score));
                        all_events.extend(ctx.events);
                    }
                }
            }
        }

        (scores, all_events, dead_agents)
    }

    fn distribute_energy_by_scores(
        &self,
        multiverse: &mut Multiverse,
        scores: Vec<(AgentId, f32)>,
        total_energy: f32,
    ) {
        let total_score: f32 = scores.iter().map(|(_, s)| s).sum();

        if total_score > 0.0 {
            for (agent_id, score) in scores {
                let proportion = score / total_score;
                let reward = proportion * total_energy;
                
                // O(1) lookup to apply reward
                if let Some(comm_id) = multiverse.agent_locations.get(&agent_id) {
                    if let Some(comm) = multiverse.spaces.get_mut(comm_id) {
                        if let Some(agent) = comm.agents.get_mut(&agent_id) {
                            agent.energy.0 += reward;
                        }
                    }
                }
            }
        }
    }
}
