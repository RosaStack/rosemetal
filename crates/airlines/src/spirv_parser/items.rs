#[derive(Debug, Default, Clone)]
pub struct SpirVSignature {
    magic: u32,
    version: (u8, u8),
}
