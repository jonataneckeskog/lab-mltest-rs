#[repr(u8)]
pub enum OPCode {
    NoOP = 0x0,   // No Operation
    AddImm = 0x1, // Add Immediate to acc
    AddMem = 0x2, // Add Memory[mp] to acc
    Jump = 0x12,  // Jump with offset of +-128
    BGT = 0x15,   //
}
