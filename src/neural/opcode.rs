#[repr(u8)]
pub enum OPCode {
    // Pipe
    NoOP = 0, // No Operation
    Jump = 1, // Jump with offset of +-128

    // Arithmetic
    AddImm = 2,    // Add Immediate to acc
    XorImm = 3,    // Xor Immediate with acc
    AndImm = 4,    // And Immediate with acc
    OrImm = 5,     // Or Immediate with acc
    ShiftImm = 6,  // Shift Immediate with acc
    RotateImm = 7, // Rotate Immediate with acc

    AddBase = 8,     // Add Memory[mp] to acc
    XorBase = 16,    // Xor Memory[mp] with acc
    AndBase = 24,    // And Memory[mp] with acc
    OrBase = 32,     // Or Memory[mp] with acc
    ShiftBase = 40,  // Shift Memory[mp] with acc
    RotateBase = 48, // Rotate Memory[mp] with acc

    // Control flow
    BGTImm = 70, // Jumps with offset if acc > Immediate
    BEImm = 71,  // Jumps with offset if acc == Immediate

    BGTBase = 64, // Jumps with offset if acc > Memory[mp]
    BEBase = 72,  // Jumps with offset if acc == Memory[mp]

    // Meta
    Not = 100, // Reverses the operation of the next instruction
}
