use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

use super::multiverse::{Community, Multiverse};
use crate::neural::{AgentManifest, BankManifest, SharedBanks};

#[derive(Serialize, Deserialize)]
pub struct CommunityManifest {
    pub agents: Vec<AgentManifest>,
    pub shared_banks: BankManifest,
    pub id: usize,
}

#[derive(Serialize, Deserialize)]
pub struct MultiverseManifest {
    pub communities: Vec<CommunityManifest>,
}

impl Community {
    pub fn save(&self, id: usize, folder: &Path) -> std::io::Result<CommunityManifest> {
        let agents: Vec<AgentManifest> = self
            .agents
            .iter()
            .enumerate()
            .map(|(id, agent)| agent.save(&id.to_string(), folder))
            .collect::<std::io::Result<Vec<_>>>()?;

        let shared_banks = self
            .shared_comms
            .save(&format!("shared_banks_{}", id), folder)?;

        Ok(CommunityManifest {
            agents,
            shared_banks,
            id,
        })
    }
}

impl CommunityManifest {
    pub fn load(&self, folder: &Path) -> anyhow::Result<Community> {
        let agents = self
            .agents
            .iter()
            .map(|agent_manifest| agent_manifest.load(folder))
            .collect::<anyhow::Result<Vec<_>>>()?;

        let shared_comms: SharedBanks = self.shared_banks.load(folder)?;

        Ok(Community {
            agents,
            shared_comms,
        })
    }
}

impl Multiverse {
    pub fn save(&self, folder: &Path) -> std::io::Result<MultiverseManifest> {
        let communities = self
            .spaces
            .iter()
            .map(|(id, community)| {
                community
                    .save(*id, folder)
                    .expect("Failed to save community")
            })
            .collect();

        Ok(MultiverseManifest { communities })
    }
}

impl MultiverseManifest {
    pub fn load(&self, folder: &Path) -> anyhow::Result<Multiverse> {
        let mut spaces = HashMap::new();
        for community_manifest in &self.communities {
            let community = community_manifest.load(folder)?;
            spaces.insert(community_manifest.id, community);
        }
        Ok(Multiverse { spaces })
    }
}

#[cfg(test)]
mod tests {
    use crate::neural::SharedBanks;
}
