use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    multiverse::{Community, Multiverse},
    neural::{
        load_agent_binary, load_shared_banks_binary, save_agent_binary, save_shared_banks_binary,
    },
};

// ---------------------------------------------------------
// 1. JSON-Friendly Save States
// ---------------------------------------------------------

#[derive(Serialize, Deserialize)]
pub struct MultiverseSaveState {
    pub spaces: HashMap<usize, CommunitySaveState>,
}

#[derive(Serialize, Deserialize)]
pub struct CommunitySaveState {
    pub agent_files: Vec<String>, // e.g., ["agents/agent_0.bin", "agents/agent_1.bin"]
    pub shared_banks_file: String, // e.g., "shared/community_5_banks.bin"
}

// ---------------------------------------------------------
// 2. Top-Level API
// ---------------------------------------------------------

/// Saves the entire Multiverse into a target directory.
/// Creates the directory if it doesn't exist.
pub fn save_multiverse(
    multiverse: &Multiverse,
    dir_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = Path::new(dir_path);
    std::fs::create_dir_all(base_dir)?; // Ensure the folder exists

    let mut mv_state = MultiverseSaveState {
        spaces: HashMap::new(),
    };

    for (&space_id, community) in &multiverse.spaces {
        let mut comm_state = CommunitySaveState {
            agent_files: Vec::new(),
            shared_banks_file: format!("community_{}_banks.bin", space_id),
        };

        // 1. Save SharedBanks
        let banks_path = base_dir.join(&comm_state.shared_banks_file);
        save_shared_banks_binary(&community.shared_comms, banks_path.to_str().unwrap())?;

        // 2. Save all Agents in this community
        for (idx, agent) in community.agents.iter().enumerate() {
            let agent_filename = format!("community_{}_agent_{}.bin", space_id, idx);
            let agent_path = base_dir.join(&agent_filename);

            save_agent_binary(agent, agent_path.to_str().unwrap())?;
            comm_state.agent_files.push(agent_filename);
        }

        // 3. Register community to the manifest
        mv_state.spaces.insert(space_id, comm_state);
    }

    // 4. Finally, write the human-readable manifest
    let manifest_path = base_dir.join("manifest.json");
    let json_string = serde_json::to_string_pretty(&mv_state)?;
    std::fs::write(manifest_path, json_string)?;

    Ok(())
}

/// Rebuilds the Multiverse from a target directory
pub fn load_multiverse(dir_path: &str) -> Result<Multiverse, Box<dyn std::error::Error>> {
    let base_dir = Path::new(dir_path);

    // 1. Read the manifest
    let manifest_path = base_dir.join("manifest.json");
    let json_string = std::fs::read_to_string(manifest_path)?;
    let mv_state: MultiverseSaveState = serde_json::from_str(&json_string)?;

    let mut multiverse = Multiverse {
        spaces: HashMap::new(),
    };

    // 2. Rebuild each community
    for (space_id, comm_state) in mv_state.spaces {
        // Load Banks
        let banks_path = base_dir.join(&comm_state.shared_banks_file);
        let shared_comms = load_shared_banks_binary(banks_path.to_str().unwrap())?;

        // Load Agents
        let mut agents = Vec::with_capacity(comm_state.agent_files.len());
        for agent_file in comm_state.agent_files {
            let agent_path = base_dir.join(agent_file);
            let agent = load_agent_binary(agent_path.to_str().unwrap())?;
            agents.push(agent);
        }

        multiverse.spaces.insert(
            space_id,
            Community {
                agents,
                shared_comms,
            },
        );
    }

    Ok(multiverse)
}
