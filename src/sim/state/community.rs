use crate::{
    core::AgentId,
    neural::{Agent, SharedBanks},
};
use std::collections::HashMap;

pub struct Community {
    pub agents: HashMap<AgentId, Agent>,
    pub shared_comms: SharedBanks,
}

impl Community {
    pub fn new() -> Self {
        Community {
            agents: HashMap::new(),
            shared_comms: SharedBanks::default(),
        }
    }
}

impl Default for Community {
    fn default() -> Self {
        Self::new()
    }
}
