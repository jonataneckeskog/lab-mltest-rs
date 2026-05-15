use crate::{
    core::{AgentId, CommunityId, MultiStepTask},
    neural::{Agent, AgentVmMemory, SharedBanks},
    sim::core::{SimulationContext, SimulationEvent},
    sim::runner::engine::SimulationRunner,
    vm::TerminationReason,
};

/// A Session provides a direct interface to manipulate and run a single agent
/// within the context of its community and the multiverse.
pub struct AgentSession<'a> {
    pub agent: &'a mut Agent,
    pub community_id: CommunityId,
    pub shared_banks: &'a mut SharedBanks,
    pub events: Vec<SimulationEvent>,
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

    pub fn from_multiverse(
        multiverse: &'a mut crate::sim::state::Multiverse,
        comm_id: CommunityId,
        agent_id: AgentId,
    ) -> anyhow::Result<Self> {
        let community = multiverse
            .spaces
            .get_mut(&comm_id)
            .ok_or_else(|| anyhow::anyhow!("Community not found"))?;
        let agent = community
            .agents
            .get_mut(&agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found"))?;

        Ok(Self::new(
            agent,
            comm_id,
            &mut community.shared_comms,
        ))
    }
}

impl<'a> SimulationRunner<'a> {
    /// Run a multi-step task for a specific agent session.
    /// Rewards are applied immediately to the agent's energy at each step.
    pub fn run_multi(
        &self,
        session: &mut AgentSession,
        task: &mut dyn MultiStepTask,
        step_energy_budget: f32,
        max_steps_per_tick: usize,
    ) {
        while !task.is_finished() {
            session.agent.load_input(task.next_input());

            let mut ctx = SimulationContext::new(AgentId(0), session.community_id);
            let summary = {
                let mut memory = AgentVmMemory::new(session.agent, session.shared_banks);
                self.executor.run(&mut memory, &mut ctx, max_steps_per_tick)
            };
            session.events.extend(ctx.events);

            match summary.reason {
                TerminationReason::Died => {
                    session.agent.energy.0 = 0.0;
                    return; // Stop immediately on death
                }
                _ => {
                    let output = session.agent.collect_output();
                    let performance = task.step(&output).max(0.0);
                    // Immediate reward injection
                    session.agent.energy.0 += performance * step_energy_budget;
                }
            }
        }
    }
}
