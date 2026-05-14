use crate::{
    neural::{Agent, AgentId, AgentSpawner, SharedBanks},
    sim::engine::SimulationEvent,
    sim::runner::AgentSession,
    sim::storage::CommunityId,
};
use std::collections::HashMap;

pub struct Multiverse {
    pub(crate) spaces: HashMap<CommunityId, Community>,
}

pub struct Community {
    pub(crate) agents: HashMap<AgentId, Agent>,
    pub(crate) shared_comms: SharedBanks,
}

impl Multiverse {
    pub fn new() -> Self {
        Multiverse {
            spaces: HashMap::new(),
        }
    }

    /// Create a randomly initialized multiverse with X communities and N agents per community.
    pub fn new_random(
        rng: &mut impl rand::Rng,
        num_communities: usize,
        agents_per_community: usize,
        spawn_energy: f32,
    ) -> Self {
        let mut multiverse = Self::new();
        let spawner = AgentSpawner { spawn_energy };

        for i in 0..num_communities {
            let mut community = Community::new();
            for _ in 0..agents_per_community {
                community.add_agent(spawner.new_random(rng));
            }
            multiverse.add_community(CommunityId(i), community);
        }

        multiverse
    }

    pub fn add_community(&mut self, id: CommunityId, community: Community) {
        self.spaces.insert(id, community);
    }

    pub fn survivor_count(&self) -> usize {
        self.spaces.values().map(|c| c.agents.len()).sum()
    }

    /// Obtain a session for a specific agent.
    pub fn session(
        &mut self,
        comm_id: CommunityId,
        agent_id: AgentId,
    ) -> anyhow::Result<AgentSession<'_>> {
        let community = self
            .spaces
            .get_mut(&comm_id)
            .ok_or_else(|| anyhow::anyhow!("Community not found"))?;
        let agent = community
            .agents
            .get_mut(&agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found"))?;

        Ok(AgentSession::new(
            agent,
            comm_id,
            &mut community.shared_comms,
        ))
    }

    pub fn resolve_events(&mut self, events: Vec<SimulationEvent>) {
        for event in events {
            match event {
                SimulationEvent::LeaveCommunity {
                    agent_id,
                    target_community_id,
                } => {
                    // Find which community the agent is currently in
                    let source_id = self.spaces.iter().find_map(|(id, comm)| {
                        if comm.agents.contains_key(&agent_id) {
                            Some(*id)
                        } else {
                            None
                        }
                    });

                    if let Some(from_id) = source_id {
                        let _ = self.migrate_agent(from_id, target_community_id, agent_id);
                    }
                }
                SimulationEvent::SpawnChild { parent_id, energy } => {
                    // Simplified: Spawn child in the same community as parent
                    let source_comm = self
                        .spaces
                        .values_mut()
                        .find(|comm| comm.agents.contains_key(&parent_id));

                    if let Some(comm) = source_comm {
                        let can_spawn = if let Some(parent) = comm.agents.get(&parent_id) {
                            parent.energy.0 > energy
                        } else {
                            false
                        };

                        if can_spawn {
                            let parent = comm.agents.get(&parent_id).unwrap();
                            let spawner = AgentSpawner {
                                spawn_energy: energy,
                            };
                            let child = spawner.spawn_standard(parent);

                            // Deduct energy from parent
                            comm.agents.get_mut(&parent_id).unwrap().energy.0 -= energy;

                            comm.add_agent(child);
                        }
                    }
                }
            }
        }
    }

    pub fn remove_agent(&mut self, comm_id: CommunityId, agent_id: AgentId) {
        if let Some(community) = self.spaces.get_mut(&comm_id) {
            community.agents.remove(&agent_id);
        }
    }

    pub fn migrate_agent(
        &mut self,
        from_id: CommunityId,
        to_id: CommunityId,
        agent_id: AgentId,
    ) -> anyhow::Result<()> {
        let agent = self
            .spaces
            .get_mut(&from_id)
            .ok_or_else(|| anyhow::anyhow!("Source community not found"))?
            .agents
            .remove(&agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found in source community"))?;

        self.spaces
            .get_mut(&to_id)
            .ok_or_else(|| anyhow::anyhow!("Destination community not found"))?
            .agents
            .insert(agent_id, agent);

        Ok(())
    }
}

impl Community {
    pub fn new() -> Self {
        Community {
            agents: HashMap::new(),
            shared_comms: SharedBanks::default(),
        }
    }

    pub fn add_agent(&mut self, agent: Agent) -> AgentId {
        let mut candidate = AgentId(0);
        while self.agents.contains_key(&candidate) {
            candidate.0 += 1;
        }
        self.agents.insert(candidate, agent);
        candidate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neural::Agent;
    use crate::sim::runner::SimulationRunner;
    use crate::sim::task::SingleStepTask;
    use crate::vm::AgentExecutor;

    #[test]
    fn test_add_agent_assigns_zero_for_empty_community() {
        let mut community = Community::new();
        let id = community.add_agent(Agent::default());

        assert_eq!(id, AgentId(0));
        assert!(community.agents.contains_key(&id));
        assert_eq!(community.agents.len(), 1);
    }

    #[test]
    fn test_add_agent_uses_nonexistent_id_when_gap_exists() {
        let mut community = Community::new();
        community.agents.insert(AgentId(0), Agent::default());
        community.agents.insert(AgentId(2), Agent::default());

        let id = community.add_agent(Agent::default());

        assert_eq!(id, AgentId(1));
        assert!(community.agents.contains_key(&id));
        assert_eq!(community.agents.len(), 3);
    }

    struct MockTask;
    impl SingleStepTask for MockTask {
        fn input_data(&self) -> &[u8] {
            &[]
        }
        fn evaluate(&self, output: &[u8]) -> f32 {
            if output.is_empty() {
                0.0
            } else {
                output[0] as f32
            }
        }
    }

    #[test]
    fn test_step_population_energy_distribution() {
        let mut multiverse = Multiverse::new();
        let mut comm = Community::new();

        let mut a1 = Agent::default();
        a1.genome = vec![0]; // Dummy genome
        let mut a2 = Agent::default();
        a2.genome = vec![0];

        // Manually set "output" via private banks for the mock task to read
        // Note: Agent::collect_output reads from bank 1.
        a1.private_banks.raw_mut(1)[0] = 1; // len
        a1.private_banks.raw_mut(1)[1] = 10; // score 10

        a2.private_banks.raw_mut(1)[0] = 1; // len
        a2.private_banks.raw_mut(1)[1] = 30; // score 30

        comm.agents.insert(AgentId(1), a1);
        comm.agents.insert(AgentId(2), a2);
        multiverse.add_community(CommunityId(1), comm);

        let executor = AgentExecutor::new(&[0.0; 256]);
        let runner = SimulationRunner::new(&executor);

        // Total score = 10 + 30 = 40.
        // Budget = 100.
        // a1 (score 10) -> 10/40 * 100 = 25
        // a2 (score 30) -> 30/40 * 100 = 75
        let rng = &mut rand::rng();
        runner.run_population_tick(rng, &mut multiverse, &MockTask, 100.0, 0);

        let comm_ref = multiverse.spaces.get(&CommunityId(1)).unwrap();

        assert_eq!(comm_ref.agents.get(&AgentId(1)).unwrap().energy.0, 25.0);
        assert_eq!(comm_ref.agents.get(&AgentId(2)).unwrap().energy.0, 75.0);
    }
}
