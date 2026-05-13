use crate::neural::{
    agent::Agent,
    agent_memory::{Bank, BankMetadata},
};

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct AgentManifest {
    pub energy: f32,
    pub genome_path: PathBuf,
    pub banks: BankManifest,
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
    pub fn save(&self, id: &str, folder: &Path) -> std::io::Result<BankManifest> {
        let filename = format!("{}_{}.bin", Self::PREFIX, id);
        // ... identical serialization logic as above ...
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
    pub fn save(&self, id: &str, folder: &Path) -> std::io::Result<AgentManifest> {
        let genome_filename = format!("genome_{}.bin", id);
        std::fs::write(folder.join(&genome_filename), &self.genome)?;

        let bank_manifest = self.private_banks.save(id, folder)?;

        Ok(AgentManifest {
            energy: self.energy.0,
            genome_path: PathBuf::from(genome_filename),
            banks: bank_manifest,
        })
    }
}

impl AgentManifest {
    pub fn load(&self, base_dir: &Path) -> anyhow::Result<Agent> {
        let genome = std::fs::read(base_dir.join(&self.genome_path))?;

        let banks = self.banks.load(base_dir)?;

        Ok(Agent {
            genome,
            energy: OrderedFloat(self.energy),
            private_banks: banks,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
