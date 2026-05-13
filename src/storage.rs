use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    multiverse::{Community, Multiverse},
    neural::AgentManifest,
};

#[derive(Serialize, Deserialize)]
pub struct CommunityManifest {
    pub agents: Vec<AgentManifest>,
    pub shared_banks_path: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct MultiverseManifest {
    pub communities: Vec<PathBuf>, // Paths to the community_X.json files
}

#[cfg(test)]
mod tests {
    use crate::neural::SharedBanks;
}
