use crate::neural::opcode::op;

macro_rules! define_costs {
    (default: $default:expr, { $($op:expr => $cost:expr),* $(,)? }) => {{
        let mut costs = [$default; 256];
        $( costs[$op as usize] = $cost; )*
        costs
    }};
}

pub const OP_COSTS: [f32; 256] = {
    // Basic arithmetic & stack ops
    let mut costs = define_costs!(default: 0.01, {
        // Tier 0: Negligible
        op::HALT         => 0.0,
        op::NO_OP        => 0.001,

        // Tier 2: Control Flow (Slightly more expensive due to branching)
        op::JUMP         => 0.015,
        op::JUMP_IF      => 0.02,
        op::JUMP_IF_NOT  => 0.02,
        op::CALL         => 0.03,   // CALL is slightly more expensive due to call stack management
        op::CALL_IND     => 0.03,
        op::RET          => 0.06,   // RET is expensive due to return address handling

        // Tier 3: Complex Math
        op::MUL          => 0.04,
        op::DIV          => 0.07,
        op::MOD          => 0.07,

        // Tier 4: System/Special
        op::REF_IND      => 0.08,   // Self-mutation/Genome rewriting
        op::RNG          => 0.03,   // Pseudo-random RNG is somewhat encouraged
    });

    // Bank Ranges (Chunked instructions with varying costs based on size)
    let mut i = 0;
    while i < 8 {
        let idx = i as u8;
        // Direct access (Genome-defined address)
        costs[(op::LOAD_BASE + idx) as usize] = 0.02;
        costs[(op::STORE_BASE + idx) as usize] = 0.02;
        costs[(op::LOADC_BASE + idx) as usize] = 0.02;
        costs[(op::STOREC_BASE + idx) as usize] = 0.02;

        // Indirect access (Stack-defined address - more pops = more cost)
        costs[(op::LOAD_IND_BASE + idx) as usize] = 0.03;
        costs[(op::STORE_IND_BASE + idx) as usize] = 0.03;
        costs[(op::LOADC_IND_BASE + idx) as usize] = 0.03;
        costs[(op::STOREC_IND_BASE + idx) as usize] = 0.03;

        i += 1;
    }

    costs
};
