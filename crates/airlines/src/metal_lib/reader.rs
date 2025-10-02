use anyhow::{Result, anyhow};

use crate::metal_lib::{MtlLibrary, MtlLibraryPlatform, MtlLibraryTargetOS, MtlLibraryType};

impl MtlLibrary {
    pub fn read(&mut self, content: &[u8]) -> Result<&mut Self> {
        self.content = content.to_vec();

        self.read_signature()?;

        Ok(self)
    }

    pub fn read_signature(&mut self) -> Result<()> {
        let start = [
            self.advance()?,
            self.advance()?,
            self.advance()?,
            self.advance()?,
        ];

        if start != [b'M', b'T', b'L', b'B'] {
            return Err(anyhow!("This file is not a Metal Library/Binary."));
        }

        let target_platform =
            MtlLibraryPlatform::from_u16(u16::from_le_bytes([self.advance()?, self.advance()?]))?;

        let version = (
            u16::from_le_bytes([self.advance()?, self.advance()?]),
            u16::from_le_bytes([self.advance()?, self.advance()?]),
        );

        let library_type = MtlLibraryType::from_u8(self.advance()?)?;

        let target_os = MtlLibraryTargetOS::from_integers(
            self.advance()?,
            u16::from_le_bytes([self.advance()?, self.advance()?]),
            u16::from_le_bytes([self.advance()?, self.advance()?]),
        )?;

        todo!("{:?}", target_os)
    }

    pub fn advance(&mut self) -> Result<u8> {
        self.position += 1;

        if self.position - 1 > self.content.len() {
            return Err(anyhow!("Position out of bounds."));
        }

        Ok(self.content[self.position - 1])
    }
}
