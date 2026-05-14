use crate::{
    neural::{Agent, AgentId, AgentSpawner, SharedBanks},
    sim::engine::SimulationEvent,
    sim::runner::AgentSession,
    sim::storage::CommunityId,
};
use std::collections::HashMap;

pub struct Multiverse {
    pub(crate) spaces: HashMap<CommunityId, Community>,
    pub(crate) population: usize,
}

pub struct Community {
    pub(crate) agents: HashMap<AgentId, Agent>,
    pub(crate) shared_comms: SharedBanks,
}

impl Multiverse {
    pub fn new() -> Self {
        Multiverse {
            spaces: HashMap::new(),
            population: 0,
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
            for _ in 0..agents_per_community {
                let agent = spawner.new_random(rng);
                multiverse.add_agent_to_community(CommunityId(i), agent);
            }
        }

        multiverse
    }

    pub fn add_community(&mut self, id: CommunityId, community: Community) {
        self.population += community.agents.len();
        self.spaces.insert(id, community);
    }

    /// Add an agent to a community and track population.
    pub fn add_agent_to_community(&mut self, comm_id: CommunityId, agent: Agent) -> AgentId {
        let community = self.spaces.entry(comm_id).or_insert_with(Community::new);
        let agent_id = community.add_agent(agent);
        self.population += 1;
        agent_id
    }

    /// Add an agent to a random community without requiring RNG.
    pub fn add_agent_to_random_community(&mut self, agent: Agent) -> AgentId {
        let comm_id = if self.spaces.is_empty() {
            CommunityId(0)
        } else {
            *self.spaces.keys().next().unwrap()
        };
        self.add_agent_to_community(comm_id, agent)
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
                    let (source_comm_id, child) = self
                        .spaces
                        .iter_mut()
                        .find_map(|(comm_id, comm)| {
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

                                Some((*comm_id, child))
                            } else {
                                None
                            }
                        })
                        .unzip();

                    if let (Some(comm_id), Some(child)) = (source_comm_id, child) {
                        self.add_agent_to_community(comm_id, child);
                    }
                }
            }
        }
    }

    pub fn remove_agent(&mut self, comm_id: CommunityId, agent_id: AgentId) -> Option<Agent> {
        let comm = self.spaces.get_mut(&comm_id)?;
        let agent = comm.agents.remove(&agent_id)?;
        if comm.agents.is_empty() {
            self.spaces.remove(&comm_id);
        }
        self.population = self.population.saturating_sub(1);
        Some(agent)
    }

    pub fn migrate_agent(
        &mut self,
        from_id: CommunityId,
        to_id: CommunityId,
        agent_id: AgentId,
    ) -> anyhow::Result<()> {
        let agent = self
            .remove_agent(from_id, agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found in source community"))?;

        self.add_agent_to_community(to_id, agent);

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
}
