use crate::core::{AgentId, CommunityId};
use crate::vm::{ByteStack, VmContext, op};

pub struct SimulationContext {
    pub agent_id: AgentId,
    pub community_id: CommunityId,
    pub events: Vec<SimulationEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimulationEvent {
    LeaveCommunity {
        agent_id: AgentId,
        target_community_id: CommunityId,
    },
    SpawnChild {
        parent_id: AgentId,
        energy: f32,
    },
}

impl SimulationContext {
    pub fn new(agent_id: AgentId, community_id: CommunityId) -> Self {
        Self {
            agent_id,
            community_id,
            events: Vec::new(),
        }
    }
}

impl VmContext for SimulationContext {
    fn agent_id(&self) -> u8 {
        (self.agent_id.0 & 0xFF) as u8
    }

    fn community_id(&self) -> u8 {
        (self.community_id.0 & 0xFF) as u8
    }

    fn random_byte(&self, seed: usize) -> u8 {
        // A simple high-speed bit-mixer for RNG
        let mut val = seed;
        val = (val ^ (val >> 16)).wrapping_mul(0x85ebca6b);
        val = (val ^ (val >> 13)).wrapping_mul(0xc2b2ae35);
        val ^= val >> 16;
        val as u8
    }

    fn syscall(&mut self, opcode: u8, stack: &mut ByteStack) -> bool {
        match opcode {
            op::LEAVE_COMMUNITY => {
                let target_id = stack.pop();
                self.events.push(SimulationEvent::LeaveCommunity {
                    agent_id: self.agent_id,
                    target_community_id: CommunityId(target_id as u64),
                });
                true
            }
            op::SPAWN_CHILD => {
                let val_u8 = stack.pop();
                let energy = (val_u8 as f32) * 0.39215686; // Scale 0-255 to 0-100
                self.events.push(SimulationEvent::SpawnChild {
                    parent_id: self.agent_id,
                    energy,
                });
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::ByteStack;
    use crate::vm::op;

    #[test]
    fn test_leave_community_syscall() {
        let mut ctx = SimulationContext::new(AgentId(10), CommunityId(1));
        let mut stack = ByteStack::new();
        stack.push(5); // Target community ID

        let result = ctx.syscall(op::LEAVE_COMMUNITY, &mut stack);

        assert!(result);
        assert_eq!(ctx.events.len(), 1);
        if let SimulationEvent::LeaveCommunity {
            agent_id,
            target_community_id,
        } = &ctx.events[0]
        {
            assert_eq!(agent_id.0, 10);
            assert_eq!(target_community_id.0, 5);
        } else {
            panic!("Expected LeaveCommunity event");
        }
    }

    #[test]
    fn test_spawn_child_syscall() {
        let mut ctx = SimulationContext::new(AgentId(10), CommunityId(1));
        let mut stack = ByteStack::new();
        stack.push(100); // Energy value (will be scaled)

        let result = ctx.syscall(op::SPAWN_CHILD, &mut stack);

        assert!(result);
        assert_eq!(ctx.events.len(), 1);
        if let SimulationEvent::SpawnChild { parent_id, energy } = &ctx.events[0] {
            assert_eq!(parent_id.0, 10);
            assert!((*energy - (100.0 * 0.39215686)).abs() < 0.0001);
        } else {
            panic!("Expected SpawnChild event");
        }
    }
}
