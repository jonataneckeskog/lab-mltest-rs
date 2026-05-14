use crate::neural::AgentId;
use crate::vm::{ByteStack, VmContext, op};
use crate::sim::storage::CommunityId;

pub struct SimulationContext {
    pub agent_id: AgentId,
    pub community_id: CommunityId,
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
                let _community_id = stack.pop();
                // TODO: Perform migration immediately (requires access to Multiverse)
                true
            }
            op::SPAWN_CHILD => {
                let val_u8 = stack.pop();
                let _energy = (val_u8 as f32) * 0.39215686; // Scale 0-255 to 0-100
                // TODO: Spawn child immediately (requires access to Multiverse and parent genome)
                true
            }
            _ => false
        }
    }
}
