use crate::neural::{
    agent::Agent,
    genome::Genome,
    memory::{Bank, BankMetadata},
};

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId(pub usize);

#[derive(Serialize, Deserialize)]
pub struct AgentManifest {
    pub energy: f32,
    pub base_genome_path: PathBuf,
    pub genome_path: PathBuf,
    pub banks: BankManifest,
    pub id: AgentId,
}

#[derive(Serialize, Deserialize)]
pub struct BankManifest {
    pub raw_data_path: PathBuf,
    pub bank_count: usize,
}

impl<const N: usize> Bank<N>
where
    Self: BankMetadata,
{
    pub fn save(&self, id: usize, folder: &Path) -> std::io::Result<BankManifest> {
        let filename = format!("{}_{}.bin", Self::PREFIX, id);
        let path = folder.join(&filename);
        let bytes = bincode::serialize(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(&path, bytes)?;

        Ok(BankManifest {
            raw_data_path: PathBuf::from(filename),
            bank_count: N,
        })
    }
}

impl BankManifest {
    pub fn load<const N: usize>(&self, folder: &Path) -> anyhow::Result<Bank<N>> {
        let path = folder.join(&self.raw_data_path);
        let bytes = std::fs::read(path)?;
        let banks: Bank<N> = bincode::deserialize(&bytes)?;

        if banks.0.len() != self.bank_count {
            anyhow::bail!(
                "Bank count mismatch: expected {}, found {}",
                N,
                self.bank_count
            );
        }

        Ok(banks)
    }
}

impl Agent {
    pub fn save(&self, id: AgentId, folder: &Path) -> std::io::Result<AgentManifest> {
        let genome_filename = format!("genome_{}.bin", id.0);
        let base_genome_filename = format!("base_genome_{}.bin", id.0);
        std::fs::write(folder.join(&genome_filename), &self.genome)?;
        std::fs::write(
            folder.join(&base_genome_filename),
            self.base_genome.0.as_ref(),
        )?;
        let bank_manifest = self.private_banks.save(id.0, folder)?;

        Ok(AgentManifest {
            energy: self.energy.0,
            genome_path: PathBuf::from(genome_filename),
            base_genome_path: PathBuf::from(base_genome_filename),
            banks: bank_manifest,
            id: id,
        })
    }
}

impl AgentManifest {
    pub fn load(&self, base_dir: &Path) -> anyhow::Result<Agent> {
        let genome = std::fs::read(base_dir.join(&self.genome_path))?;
        let base_genome = std::fs::read(base_dir.join(&self.base_genome_path))?;

        let banks = self.banks.load(base_dir)?;

        Ok(Agent {
            genome: genome,
            base_genome: Genome::new(base_genome),
            private_banks: banks,
            energy: OrderedFloat(self.energy),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neural::memory::{PrivateBanks, SharedBanks};
    use tempfile::tempdir;

    // Helper to create a basic agent
    fn create_test_agent() -> Agent {
        let mut agent = Agent::default();
        agent.genome = vec![1, 2, 3, 4];
        agent.energy = OrderedFloat(50.5);
        agent
    }

    #[test]
    fn test_bank_save_load_cycle() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let banks = PrivateBanks::default();

        let manifest = banks.save(0, dir.path())?;
        let loaded_banks: PrivateBanks = manifest.load(dir.path())?;

        assert_eq!(banks, loaded_banks);
        Ok(())
    }

    #[test]
    fn test_bank_count_mismatch_fails() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let private = PrivateBanks::default();
        let manifest = private.save(1, dir.path())?;

        // Try to load 6 banks into a struct expecting 2 (SharedBanks)
        let result = manifest.load::<2>(dir.path());
        assert!(result.is_err(), "Should fail when bank counts don't match");
        Ok(())
    }

    #[test]
    fn test_private_banks_io_persistence() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let mut banks = PrivateBanks::default();
        banks.write_input(&[10, 20, 30]);

        let manifest = banks.save(2, dir.path())?;
        let loaded = manifest.load::<6>(dir.path())?;

        assert_eq!(loaded.read_output(), Vec::<u8>::new()); // Output bank should still be empty
        // Re-verify input was saved
        assert_eq!(loaded.0[0][1], 10);
        Ok(())
    }

    #[test]
    fn test_agent_save_creates_files() -> std::io::Result<()> {
        let dir = tempdir()?;
        let agent = create_test_agent();

        agent.save(AgentId(1), dir.path())?;

        assert!(dir.path().join("genome_1.bin").exists());
        assert!(dir.path().join("private_banks_1.bin").exists());
        Ok(())
    }

    #[test]
    fn test_agent_full_round_trip() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let original = create_test_agent();

        let manifest = original.save(AgentId(1), dir.path())?;
        let recovered = manifest.load(dir.path())?;

        assert_eq!(original.genome, recovered.genome);
        assert_eq!(original.energy, recovered.energy);
        assert_eq!(original.private_banks, recovered.private_banks);
        Ok(())
    }

    #[test]
    fn test_manifest_serialization_itself() -> anyhow::Result<()> {
        // Tests if the AgentManifest JSON/Bincode itself is valid
        let manifest = AgentManifest {
            energy: 10.0,
            base_genome_path: PathBuf::from("bg.bin"),
            genome_path: PathBuf::from("g.bin"),
            banks: BankManifest {
                raw_data_path: PathBuf::from("b.bin"),
                bank_count: 6,
            },
            id: AgentId(1),
        };

        let encoded = bincode::serialize(&manifest)?;
        let decoded: AgentManifest = bincode::deserialize(&encoded)?;

        assert_eq!(decoded.energy, 10.0);
        assert_eq!(decoded.banks.bank_count, 6);
        Ok(())
    }

    #[test]
    fn test_save_with_nonexistent_folder_fails() {
        let agent = create_test_agent();
        let result = agent.save(AgentId(0), Path::new("/definitely/not/a/real/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_bank_raw_mut_persistence() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let mut banks = PrivateBanks::default();
        banks.raw_mut(2)[10] = 255; // Modify the 3rd bank

        let manifest = banks.save(3, dir.path())?;
        let loaded = manifest.load::<6>(dir.path())?;

        assert_eq!(loaded.0[2][10], 255);
        Ok(())
    }

    #[test]
    fn test_shared_banks_save_logic() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let shared = SharedBanks::default();

        let manifest = shared.save(0, dir.path())?;
        assert_eq!(manifest.bank_count, 2);
        assert!(dir.path().join("shared_banks_0.bin").exists());
        Ok(())
    }

    #[test]
    fn test_large_genome_persistence() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let mut agent = Agent::default();
        agent.genome = vec![0u8; 1024 * 10]; // 10kb genome

        let manifest = agent.save(AgentId(1), dir.path())?;
        let recovered = manifest.load(dir.path())?;

        assert_eq!(recovered.genome.len(), 1024 * 10);
        Ok(())
    }
}
