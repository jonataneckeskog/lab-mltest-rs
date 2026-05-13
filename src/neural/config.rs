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
        op::JUMP_IF      => 0.015,
        op::JUMP_IF_NOT  => 0.015,
        op::CALL         => 0.02,
        op::CALL_IND     => 0.025,
        op::RET          => 0.02,

        // Tier 3: Complex Math
        op::MUL          => 0.05,
        op::DIV          => 0.05,
        op::MOD          => 0.05,

        // Tier 4: System/Special (High overhead)
        op::REF_IND      => 0.5,   // Self-mutation/Genome rewriting
        op::RNG          => 0.1,   // Pseudo-random RNG is somewhat encouraged
    });

    // Bank Ranges
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
