use crate::neural::opcode::op;

macro_rules! define_costs {
    (default: $default:expr, { $($op:expr => $cost:expr),* $(,)? }) => {{
        let mut costs = [$default; 256];
        $( costs[$op as usize] = $cost; )*
        costs
    }};
}

pub const OP_COSTS: [f32; 256] = define_costs!(default: 0.01, {
    op::NO_OP   => 0.001,
    op::HALT    => 0.0,
    op::MUL     => 0.05,
    op::DIV     => 0.05,
    op::REF_IND => 0.5,
});
