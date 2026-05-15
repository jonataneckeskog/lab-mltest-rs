use crate::sim::state::community::Community;
use crate::{
    core::{AgentId, CommunityId},
    neural::{Agent, AgentSpawner},
};
use rand::RngExt;
use std::collections::HashMap;

pub struct Multiverse {
    pub spaces: HashMap<CommunityId, Community>,
    pub population: usize,
    pub agent_locations: HashMap<AgentId, CommunityId>,
    pub next_agent_id: u64,
}

impl Multiverse {
    pub fn new() -> Self {
        Multiverse {
            spaces: HashMap::new(),
            population: 0,
            agent_locations: HashMap::new(),
            next_agent_id: 0,
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
                multiverse.add_agent_to_community(CommunityId(i as u64), agent);
            }
        }

        multiverse
    }

    pub fn add_community(&mut self, id: CommunityId, community: Community) {
        for agent_id in community.agents.keys() {
            self.agent_locations.insert(*agent_id, id);
            if agent_id.0 >= self.next_agent_id {
                self.next_agent_id = agent_id.0 + 1;
            }
        }
        self.population += community.agents.len();
        self.spaces.insert(id, community);
    }

    /// Add an agent to a community and track population.
    pub fn add_agent_to_community(&mut self, comm_id: CommunityId, agent: Agent) -> AgentId {
        let agent_id = AgentId(self.next_agent_id);
        self.next_agent_id += 1;

        let community = self.spaces.entry(comm_id).or_insert_with(Community::new);
        community.agents.insert(agent_id, agent);
        self.agent_locations.insert(agent_id, comm_id);
        self.population += 1;
        agent_id
    }

    /// Internal helper to add an agent with a specific ID (used during migration/loading).
    pub(crate) fn force_add_agent(
        &mut self,
        comm_id: CommunityId,
        agent_id: AgentId,
        agent: Agent,
    ) {
        let community = self.spaces.entry(comm_id).or_insert_with(Community::new);
        community.agents.insert(agent_id, agent);
        self.agent_locations.insert(agent_id, comm_id);
        self.population += 1;
        if agent_id.0 >= self.next_agent_id {
            self.next_agent_id = agent_id.0 + 1;
        }
    }

    /// Add an agent to a random community.
    pub fn add_agent_to_random_community(
        &mut self,
        rng: &mut impl rand::Rng,
        agent: Agent,
    ) -> AgentId {
        let comm_id = if self.spaces.is_empty() {
            CommunityId(0)
        } else {
            let random_index = rng.random_range(0..self.spaces.len());

            *self.spaces.keys().nth(random_index).unwrap()
        };

        self.add_agent_to_community(comm_id, agent)
    }

    /// Returns a flattened iterator over all agents in the multiverse.
    pub fn agents(&self) -> impl Iterator<Item = (CommunityId, AgentId, &Agent)> {
        self.spaces.iter().flat_map(|(comm_id, community)| {
            community
                .agents
                .iter()
                .map(move |(agent_id, agent)| (*comm_id, *agent_id, agent))
        })
    }

    /// Returns a flattened mutable iterator over all agents in the multiverse.
    pub fn agents_mut(&mut self) -> impl Iterator<Item = (CommunityId, AgentId, &mut Agent)> {
        self.spaces.iter_mut().flat_map(|(comm_id, community)| {
            community
                .agents
                .iter_mut()
                .map(move |(agent_id, agent)| (*comm_id, *agent_id, agent))
        })
    }

    pub fn get_max_energy(&self) -> f32 {
        self.agents()
            .map(|(_, _, a)| a.energy.0)
            .fold(0.0, f32::max)
    }

    pub fn get_energy_stats(&self) -> (f32, f32, f32) {
        if self.population == 0 {
            return (0.0, 0.0, 0.0);
        }

        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        let mut sum = 0.0;

        for (_, _, agent) in self.agents() {
            let e = agent.energy.0;
            min = min.min(e);
            max = max.max(e);
            sum += e;
        }

        (min, max, sum / self.population as f32)
    }

    pub fn remove_agent(&mut self, agent_id: AgentId) -> Option<(CommunityId, Agent)> {
        let comm_id = self.agent_locations.remove(&agent_id)?;
        let comm = self.spaces.get_mut(&comm_id)?;
        let agent = comm.agents.remove(&agent_id)?;
        if comm.agents.is_empty() {
            self.spaces.remove(&comm_id);
        }
        self.population = self.population.saturating_sub(1);
        Some((comm_id, agent))
    }
}

impl Default for Multiverse {
    fn default() -> Self {
        Self::new()
    }
}
