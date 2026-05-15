use crate::core::{AgentId, CommunityId};
use crate::neural::AgentSpawner;
use crate::sim::core::events::SimulationEvent;
use crate::sim::state::multiverse::Multiverse;

pub fn resolve_events(multiverse: &mut Multiverse, events: Vec<SimulationEvent>) {
    for event in events {
        match event {
            SimulationEvent::LeaveCommunity {
                agent_id,
                target_community_id,
            } => {
                let _ = migrate_agent(multiverse, agent_id, target_community_id);
            }
            SimulationEvent::SpawnChild { parent_id, energy } => {
                if energy <= 0.0 {
                    continue;
                }

                let spawn_data = multiverse
                    .agent_locations
                    .get(&parent_id)
                    .and_then(|comm_id| {
                        let comm = multiverse.spaces.get(comm_id)?;
                        let parent = comm.agents.get(&parent_id)?;
                        if parent.energy.0 > energy {
                            let child = AgentSpawner::spawn_child(parent, energy);
                            Some((*comm_id, child))
                        } else {
                            None
                        }
                    });

                if let Some((comm_id, child)) = spawn_data {
                    if let Some(comm) = multiverse.spaces.get_mut(&comm_id) {
                        if let Some(parent) = comm.agents.get_mut(&parent_id) {
                            parent.energy.0 -= energy;
                        }
                    }
                    // Add child to the same community
                    multiverse.add_agent_to_community(comm_id, child);
                }
            }
        }
    }
}

pub fn migrate_agent(
    multiverse: &mut Multiverse,
    agent_id: AgentId,
    to_id: CommunityId,
) -> anyhow::Result<()> {
    let (_from_id, agent) = multiverse
        .remove_agent(agent_id)
        .ok_or_else(|| anyhow::anyhow!("Agent not found anywhere in the multiverse"))?;

    multiverse.force_add_agent(to_id, agent_id, agent);

    Ok(())
}

pub fn mutate_all(multiverse: &mut Multiverse, rng: &mut impl rand::Rng) {
    for (_, _, agent) in multiverse.agents_mut() {
        agent.mutate(rng);
    }
}
