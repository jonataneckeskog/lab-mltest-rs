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

        let shared_banks = self.shared_comms.save(&id.to_string(), folder)?;

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
    use super::*;
    use crate::neural::{Agent, SharedBanks};
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
        let agents = vec![create_test_agent(), create_test_agent()];
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

        community.save(42, dir.path())?;

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

        let manifest = original.save(99, dir.path())?;
        let recovered = manifest.load(dir.path())?;

        assert_eq!(original.agents.len(), recovered.agents.len());
        for (orig, rec) in original.agents.iter().zip(recovered.agents.iter()) {
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
        multiverse.spaces.insert(1, create_test_community());
        multiverse.spaces.insert(2, create_test_community());

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
    fn test_community_manifest_serialization() -> anyhow::Result<()> {
        let manifest = CommunityManifest {
            agents: vec![AgentManifest {
                energy: 10.0,
                genome_path: std::path::PathBuf::from("g.bin"),
                banks: BankManifest {
                    raw_data_path: std::path::PathBuf::from("b.bin"),
                    bank_count: 6,
                },
            }],
            shared_banks: BankManifest {
                raw_data_path: std::path::PathBuf::from("s.bin"),
                bank_count: 2,
            },
            id: 5,
        };

        let encoded = bincode::serialize(&manifest)?;
        let decoded: CommunityManifest = bincode::deserialize(&encoded)?;

        assert_eq!(decoded.id, 5);
        assert_eq!(decoded.agents.len(), 1);
        assert_eq!(decoded.shared_banks.bank_count, 2);
        Ok(())
    }
}
