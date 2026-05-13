pub mod op {
    // --- Pipe & Stack Manipulation ---
    pub const NO_OP: u8 = 0; // No Operation
    pub const HALT: u8 = 1; // Completely stops the program, confidence rating 0-255
    pub const POP: u8 = 2; // Discard the top value (a -- )
    pub const DUP: u8 = 3; // Duplicate the top value (a -- a, a)
    pub const SWAP: u8 = 4; // Swap the top two values (a, b -- b, a)
    pub const PUSH: u8 = 5; // Followed by 1 byte: Push immediate to stack ( -- imm)
    pub const OVER: u8 = 6; // Copies the second item to the top (a, b -- a, b, a)
    pub const SELECT: u8 = 7; // (val_a, val_b, cond -- result)
    pub const EXEC_STACK: u8 = 8; // Execute the top value as an opcode (val --)

    // --- Arithmetic & Logic --
    // These pop the top two values, perform math, and push the result
    pub const ADD: u8 = 10; // (a, b -- a+b)
    pub const SUB: u8 = 11; // (a, b -- a-b)
    pub const XOR: u8 = 12; // (a, b -- a^b)
    pub const AND: u8 = 13; // (a, b -- a&b)
    pub const OR: u8 = 14; // (a, b -- a|b)
    pub const NOT: u8 = 15; // Bitwise NOT (a -- !a)
    pub const SHL: u8 = 16; // Shift Left (val, amt -- res)
    pub const SHR: u8 = 17; // Shift Right (val, amt -- res)
    pub const MUL: u8 = 18; // (a, b -- a*b)
    pub const DIV: u8 = 19; // (a, b -- a/b)
    pub const MOD: u8 = 20; // (a, b -- a%b)

    // --- Memory Interaction (8 Chambers) ---
    // Followed by a one byte memory address.

    // Load: Memory[addr] -> Stack
    pub const LOAD_BASE: u8 = 24;
    pub const LOAD_END: u8 = LOAD_BASE + 7;
    pub const LOAD_IND_BASE: u8 = 32;
    pub const LOAD_IND_END: u8 = LOAD_IND_BASE + 7;

    // Store: Stack -> Memory[addr] (Copies the value)
    pub const STORE_BASE: u8 = 40;
    pub const STORE_END: u8 = STORE_BASE + 7;
    pub const STORE_IND_BASE: u8 = 48;
    pub const STORE_IND_END: u8 = STORE_IND_BASE + 7;

    // Copy chunks of data Memory[addr] Amount -> Stack (addr, amt -- stack)
    pub const LOADC_BASE: u8 = 56;
    pub const LOADC_END: u8 = LOADC_BASE + 7;
    pub const LOADC_IND_BASE: u8 = 64;
    pub const LOADC_IND_END: u8 = LOADC_IND_BASE + 7;

    // Copy chunks of data Stack Amount -> Memory[addr] (addr, amt)
    pub const STOREC_BASE: u8 = 72;
    pub const STOREC_END: u8 = STOREC_BASE + 7;
    pub const STOREC_IND_BASE: u8 = 80;
    pub const STOREC_IND_END: u8 = STOREC_IND_BASE + 7;

    // --- Control Flow ---
    pub const JUMP: u8 = 100; // Followed by 1 byte: Unconditional jump (signed offset)
    pub const JUMP_IF: u8 = 101; // Followed by 1 byte: Pop stack; if value != 0, jump.
    pub const JUMP_IF_NOT: u8 = 102; // Followed by 1 byte: Pop stack; if value == 0, jump.

    // Comparisons: Pop 2 values, push 1 (true) or 0 (false)
    pub const EQ: u8 = 103; // (a, b -- a==b)
    pub const GT: u8 = 104; // (a, b -- a>b)
    pub const LT: u8 = 105; // (a, b -- a<b)

    pub const CALL: u8 = 106; // Followed by 1 byte: Call immediate address
    pub const CALL_IND: u8 = 107; // Pop stack, call that address
    pub const RET: u8 = 108; // Pop return stack and jump back

    // --- Meta ---

    // Within agent
    pub const REF_IND: u8 = 200; // Allows the AI to rewrite itself (v, o --)
    pub const DOUBLE_SIZE: u8 = 201; // Doubles the genome size (up to a set max bytes)
    pub const HALF_SIZE: u8 = 202; // Halves the genome size (down to a set min bytes)
    pub const DIE: u8 = 203; // ( -- ) Kills the agent immediately

    // Communicating with 'the world'
    pub const LEAVE_COMMUNITY: u8 = 204; // (c --) Leaves the current community
    pub const SPAWN_CHILD: u8 = 205; // 2 bytes: (mutated genome, energy)

    // --- Self awareness ---
    pub const GET_SP: u8 = 248; // Pushes the Stack Pointer to stack
    pub const GET_PC: u8 = 249; // Pushes the Program Counter to stack
    pub const GET_ENERGY: u8 = 250; // Pushes the current Energy to stack
    pub const GET_ID: u8 = 251; // Pushes the Agent's unique ID to stack
    pub const GET_COMMUNITY_ID: u8 = 252; // Pushes the current Community ID to stack
    pub const RNG: u8 = 253; // Pushes a random byte ( -- v)
}
