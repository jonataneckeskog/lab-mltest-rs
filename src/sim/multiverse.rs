use std::collections::HashMap;

use crate::neural::{Agent, SharedBanks};

pub struct Multiverse {
    pub(crate) spaces: HashMap<usize, Community>,
}

pub struct Community {
    pub(crate) agents: Vec<Agent>,
    pub(crate) shared_comms: SharedBanks,
}

impl Multiverse {
    pub fn new() -> Self {
        Multiverse {
            spaces: HashMap::new(),
        }
    }

    pub fn add_community(&mut self, id: usize, community: Community) {
        self.spaces.insert(id, community);
    }

    pub fn migrate_agent(
        &mut self,
        from_id: usize,
        to_id: usize,
        agent_index: usize,
    ) -> anyhow::Result<()> {
        let agent = self
            .spaces
            .get_mut(&from_id)
            .ok_or_else(|| anyhow::anyhow!("Source community not found"))?
            .agents
            .remove(agent_index);

        self.spaces
            .get_mut(&to_id)
            .ok_or_else(|| anyhow::anyhow!("Destination community not found"))?
            .agents
            .push(agent);

        Ok(())
    }
}

impl Community {
    pub fn new() -> Self {
        Community {
            agents: Vec::new(),
            shared_comms: SharedBanks::default(),
        }
    }

    pub fn add_agent(&mut self, agent: Agent) {
        self.agents.push(agent);
    }
}
