#[derive(Debug, Default, Clone)]
pub struct SpirVSignature {
    pub magic_number: u32,
    pub version: (u8, u8),
    pub generator_magic_number: u32,
    pub bound: u32,
    pub reserved_instruction_schema: u32,
}
