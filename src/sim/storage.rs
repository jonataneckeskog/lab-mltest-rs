use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

use super::multiverse::{Community, Multiverse};
use crate::neural::{AgentManifest, BankManifest, SharedBanks};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommunityId(pub usize);

#[derive(Serialize, Deserialize)]
pub struct CommunityManifest {
    pub agents: Vec<AgentManifest>,
    pub shared_banks: BankManifest,
    pub id: CommunityId,
}

#[derive(Serialize, Deserialize)]
pub struct MultiverseManifest {
    pub communities: Vec<CommunityManifest>,
}

impl Community {
    pub fn save(&self, id: CommunityId, folder: &Path) -> std::io::Result<CommunityManifest> {
        let agents: Vec<AgentManifest> = self
            .agents
            .iter()
            .map(|(id, agent)| agent.save(*id, folder))
            .collect::<std::io::Result<Vec<_>>>()?;

        let shared_banks = self.shared_comms.save(id.0, folder)?;

        Ok(CommunityManifest {
            agents,
            shared_banks,
            id,
        })
    }
}

impl CommunityManifest {
    pub fn load(&self, folder: &Path) -> anyhow::Result<Community> {
        let agents: HashMap<crate::neural::AgentId, crate::neural::Agent> = self
            .agents
            .iter()
            .map(|agent_manifest| {
                agent_manifest
                    .load(folder)
                    .map(|agent| (agent_manifest.id, agent))
            })
            .collect::<anyhow::Result<HashMap<_, _>>>()?;

        let shared_comms: SharedBanks = self.shared_banks.load(folder)?;

        Ok(Community {
            agents,
            shared_comms,
        })
    }
}

impl Multiverse {
    pub fn save_to(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let folder = path.as_ref();
        if !folder.exists() {
            std::fs::create_dir_all(folder)?;
        }
        let manifest = self.save(folder)?;
        let manifest_path = folder.join("multiverse.json");
        let file = std::fs::File::create(manifest_path)?;
        serde_json::to_writer_pretty(file, &manifest)?;
        Ok(())
    }

    pub fn load_from(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let folder = path.as_ref();
        let manifest_path = folder.join("multiverse.json");
        let file = std::fs::File::open(manifest_path)?;
        let manifest: MultiverseManifest = serde_json::from_reader(file)?;
        manifest.load(folder)
    }

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
    use super::*;
    use crate::neural::{Agent, AgentId, SharedBanks};
    use ordered_float::OrderedFloat;
    use tempfile::tempdir;

    // Helper to create a basic agent for testing
    fn create_test_agent() -> Agent {
        let mut agent = Agent::default();
        agent.genome = vec![1, 2, 3, 4];
        agent.energy = OrderedFloat(50.5);
        agent
    }

    // Helper to create a basic community for testing
    fn create_test_community() -> Community {
        let mut agents = HashMap::new();
        agents.insert(AgentId(0), create_test_agent());
        agents.insert(AgentId(1), create_test_agent());
        let shared_comms = SharedBanks::default();
        Community {
            agents,
            shared_comms,
        }
    }

    #[test]
    fn test_community_save_creates_files() -> std::io::Result<()> {
        let dir = tempdir()?;
        let community = create_test_community();

        community.save(CommunityId(42), dir.path())?;

        // Check that agent files are created
        assert!(dir.path().join("genome_0.bin").exists());
        assert!(dir.path().join("private_banks_0.bin").exists());
        assert!(dir.path().join("genome_1.bin").exists());
        assert!(dir.path().join("private_banks_1.bin").exists());
        // Check shared banks file
        assert!(dir.path().join("shared_banks_42.bin").exists());
        Ok(())
    }

    #[test]
    fn test_community_full_round_trip() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let original = create_test_community();

        let manifest = original.save(CommunityId(99), dir.path())?;
        let recovered = manifest.load(dir.path())?;

        assert_eq!(original.agents.len(), recovered.agents.len());
        for (id, orig) in &original.agents {
            let rec = recovered
                .agents
                .get(id)
                .expect("Recovered community missing agent");
            assert_eq!(orig.genome, rec.genome);
            assert_eq!(orig.energy, rec.energy);
            assert_eq!(orig.private_banks, rec.private_banks);
        }
        assert_eq!(original.shared_comms, recovered.shared_comms);
        Ok(())
    }

    #[test]
    fn test_multiverse_save_load_cycle() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let mut multiverse = Multiverse {
            spaces: HashMap::new(),
        };
        multiverse
            .spaces
            .insert(CommunityId(1), create_test_community());
        multiverse
            .spaces
            .insert(CommunityId(2), create_test_community());

        let manifest = multiverse.save(dir.path())?;
        let loaded = manifest.load(dir.path())?;

        assert_eq!(multiverse.spaces.len(), loaded.spaces.len());
        for (id, orig_comm) in &multiverse.spaces {
            let loaded_comm = loaded.spaces.get(id).unwrap();
            assert_eq!(orig_comm.agents.len(), loaded_comm.agents.len());
            assert_eq!(orig_comm.shared_comms, loaded_comm.shared_comms);
        }
        Ok(())
    }

    #[test]
    fn test_empty_multiverse_round_trip() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let multiverse = Multiverse {
            spaces: HashMap::new(),
        };

        let manifest = multiverse.save(dir.path())?;
        let loaded = manifest.load(dir.path())?;

        assert_eq!(loaded.spaces.len(), 0);
        Ok(())
    }

    #[test]
    fn test_multiverse_high_level_persistence() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let mut multiverse = Multiverse::new();
        multiverse.spaces.insert(CommunityId(1), create_test_community());

        multiverse.save_to(dir.path())?;
        let loaded = Multiverse::load_from(dir.path())?;

        assert_eq!(multiverse.spaces.len(), loaded.spaces.len());
        assert!(loaded.spaces.contains_key(&CommunityId(1)));
        Ok(())
    }
}
