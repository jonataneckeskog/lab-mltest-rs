use std::fs::File;
use std::io::{Read, Write};

use crate::neural::agent::Agent;
use crate::neural::agent_memory::SharedBanks;

// ---------------------------------------------------------
// 1. Binary I/O for Agents
// ---------------------------------------------------------

pub fn save_agent_binary(agent: &Agent, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encoded: Vec<u8> = bincode::serialize(agent)?;
    let mut file = File::create(filepath)?;
    file.write_all(&encoded)?;
    Ok(())
}

pub fn load_agent_binary(filepath: &str) -> Result<Agent, Box<dyn std::error::Error>> {
    let mut file = File::open(filepath)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let agent: Agent = bincode::deserialize(&buffer)?;
    Ok(agent)
}

// ---------------------------------------------------------
// 2. Binary I/O for Shared Banks
// ---------------------------------------------------------

pub fn save_shared_banks_binary(
    banks: &SharedBanks,
    filepath: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let encoded: Vec<u8> = bincode::serialize(banks)?;
    let mut file = File::create(filepath)?;
    file.write_all(&encoded)?;
    Ok(())
}

pub fn load_shared_banks_binary(filepath: &str) -> Result<SharedBanks, Box<dyn std::error::Error>> {
    let mut file = File::open(filepath)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let banks: SharedBanks = bincode::deserialize(&buffer)?;
    Ok(banks)
}
