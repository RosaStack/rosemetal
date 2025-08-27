pub mod items;

use anyhow::{Result, anyhow};
pub use items::*;

pub struct Parser {
    position: i64,
    signature: SpirVSignature,
    content: Vec<u8>,
}

impl Parser {
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            position: -1,
            signature: SpirVSignature::default(),
            content,
        }
    }

    pub fn advance(&mut self) -> Result<u8> {
        self.position += 1;

        if self.position as usize > self.content.len() {
            return Err(anyhow!("Position is bigger than the contents length."));
        }

        Ok(self.content[self.position as usize])
    }

    pub fn start(&mut self) -> Result<()> {
        let magic_number = u32::from_le_bytes([
            self.advance()?,
            self.advance()?,
            self.advance()?,
            self.advance()?,
        ]);

        if magic_number != 0x7230203 {
            return Err(anyhow!("Invalid or corrupted magic number."));
        }

        todo!("{:#X}", magic_number)
    }
}
