use std::collections::HashMap;

use crate::neural::{agent::Agent, agent_memory::SharedBanks};

struct Multiverse {
    // Communities indexed by coordinates or a simple ID
    spaces: HashMap<usize, Community>,
}

pub struct Community {
    agents: Vec<Agent>,
    shared_comms: SharedBanks,
}
