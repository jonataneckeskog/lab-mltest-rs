use std::collections::HashMap;

use crate::{
    neural::{Agent, AgentId, SharedBanks},
    sim::storage::CommunityId,
};

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

    pub fn add_community(&mut self, id: CommunityId, community: Community) {
        self.spaces.insert(id, community);
    }

    // Formula for Balanced Complexity:
    // TotalEnergy = (TargetCPU / CurrentCPU)^p * log2(N + 1) * K
    //
    // - (TargetCPU / CurrentCPU)^p: The Governor (punishes big/slow agents via actual CPU load)
    // - log2(N + 1): The Diversity Reward (rewards having "enough" agents, then tapers off)
    // - K: Global scalar constant
    pub fn compute_global_energy(&self, current_cpu: f32, target_cpu: f32, k: f32, p: f32) -> f32 {
        let n = self.spaces.values().map(|c| c.agents.len()).sum::<usize>() as f32;

        if n == 0.0 {
            return k;
        }

        // Governor: Punishes based on actual CPU load (where 'Big' agents naturally hit harder)
        let governor = (target_cpu / current_cpu.max(0.001)).powf(p);

        // Population Reward: log2 provides a "Satiation" curve.
        // 1 -> 80 agents is a massive bonus; 80 -> 5000 is a diminishing return.
        let population_reward = (n + 1.0).log2();

        governor * population_reward * k
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
