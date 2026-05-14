use crate::{
    neural::{Agent, AgentId, AgentVmMemory, SharedBanks},
    sim::engine::{SimulationContext, SimulationEvent},
    sim::storage::CommunityId,
    sim::task::{MultiStepTask, SingleStepTask},
    vm::AgentExecutor,
};

/// A Session provides a direct interface to manipulate and run a single agent
/// within the context of its community and the multiverse.
pub struct AgentSession<'a> {
    pub agent: &'a mut Agent,
    pub community_id: CommunityId,
    pub shared_banks: &'a mut SharedBanks,
    pub events: Vec<SimulationEvent>,
}

pub struct SimulationRunner<'a> {
    pub executor: &'a AgentExecutor<'a>,
}

impl<'a> AgentSession<'a> {
    pub fn new(
        agent: &'a mut Agent,
        community_id: CommunityId,
        shared_banks: &'a mut SharedBanks,
    ) -> Self {
        Self {
            agent,
            community_id,
            shared_banks,
            events: Vec::new(),
        }
    }

    pub fn run(&mut self, executor: &AgentExecutor, max_steps: usize) {
        let mut ctx = SimulationContext::new(AgentId(0), self.community_id);
        {
            let mut memory = AgentVmMemory::new(self.agent, self.shared_banks);
            executor.run(&mut memory, &mut ctx, max_steps);
        }
        self.events.extend(ctx.events);
    }
}

impl<'a> SimulationRunner<'a> {
    pub fn new(executor: &'a AgentExecutor<'a>) -> Self {
        Self { executor }
    }

    /// Orchestrate a full population tick: run agents, distribute energy, and resolve events.
    pub fn run_population_tick(
        &self,
        multiverse: &mut crate::sim::multiverse::Multiverse,
        task: &dyn SingleStepTask,
        total_energy: f32,
        max_steps: usize,
    ) {
        let (scores, all_events) = self.run_population(&mut multiverse.spaces, task, max_steps);

        // 1. Distribute Energy
        self.distribute_energy_by_scores(&mut multiverse.spaces, scores, total_energy);

        // 2. Resolve Events
        multiverse.resolve_events(all_events);
    }

    /// Run a single step task across an entire population.
    /// Returns a list of (CommunityId, AgentId, PerformanceScore) and all triggered events.
    pub fn run_population(
        &self,
        spaces: &mut std::collections::HashMap<CommunityId, crate::sim::multiverse::Community>,
        task: &dyn SingleStepTask,
        max_steps: usize,
    ) -> (Vec<(CommunityId, AgentId, f32)>, Vec<SimulationEvent>) {
        let mut all_events = Vec::new();
        let mut scores = Vec::new();
        let input = task.input_data();

        for (comm_id, community) in spaces {
            for (agent_id, agent) in &mut community.agents {
                agent.load_input(input);

                let mut sess = AgentSession::new(agent, *comm_id, &mut community.shared_comms);
                sess.run(self.executor, max_steps);

                let output = sess.agent.collect_output();
                let score = task.evaluate(&output).max(0.0);

                scores.push((*comm_id, *agent_id, score));
                all_events.extend(sess.events);
            }
        }

        (scores, all_events)
    }

    /// Run a multi-step task for a specific agent session.
    pub fn run_multi(
        &self,
        session: &mut AgentSession,
        task: &mut dyn MultiStepTask,
        max_steps_per_tick: usize,
    ) -> f32 {
        let mut total_score = 0.0;

        while !task.is_finished() {
            session.agent.load_input(task.next_input());
            session.run(self.executor, max_steps_per_tick);

            let output = session.agent.collect_output();
            total_score += task.step(&output);
        }

        total_score
    }

    fn distribute_energy_by_scores(
        &self,
        spaces: &mut std::collections::HashMap<CommunityId, crate::sim::multiverse::Community>,
        scores: Vec<(CommunityId, AgentId, f32)>,
        total_energy: f32,
    ) {
        let total_score: f32 = scores.iter().map(|(_, _, s)| s).sum();

        if total_score > 0.0 {
            for (comm_id, agent_id, score) in scores {
                let proportion = score / total_score;
                let reward = proportion * total_energy;
                if let Some(agent) = spaces
                    .get_mut(&comm_id)
                    .and_then(|c| c.agents.get_mut(&agent_id))
                {
                    agent.energy.0 = reward;
                }
            }
        } else if !scores.is_empty() {
            let share = total_energy / scores.len() as f32;
            for (comm_id, agent_id, _) in scores {
                if let Some(agent) = spaces
                    .get_mut(&comm_id)
                    .and_then(|c| c.agents.get_mut(&agent_id))
                {
                    agent.energy.0 = share;
                }
            }
        }
    }
}
