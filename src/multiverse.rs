use std::collections::HashMap;

use crate::neural::{Agent, SharedBanks};

pub struct Multiverse {
    pub(crate) spaces: HashMap<usize, Community>,
}

pub struct Community {
    pub(crate) agents: Vec<Agent>,
    pub(crate) shared_comms: SharedBanks,
}
