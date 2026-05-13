use crate::neural::opcode::op;

// Costs for opcodes 0 to 255
pub const OP_COSTS: [f32; 256] = {
    let mut costs = [0.01; 256]; // Default cost
    costs[op::NO_OP as usize] = 0.001;
    costs[op::MUL as usize] = 0.05;
    costs[op::DIV as usize] = 0.05;
    costs[op::REF_IND as usize] = 0.5; // High cost for self-mutation
    costs
};
