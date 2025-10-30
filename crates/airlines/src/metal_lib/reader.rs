use anyhow::{Result, anyhow};

use crate::{
    air_parser,
    metal_lib::{
        MTLLibraryParser, MTLLibraryPlatform, MTLLibrarySignature, MTLLibraryTargetOS,
        MTLLibraryTargetOSType, MTLLibraryType,
    },
};

impl MTLLibraryParser {
    pub fn read(&mut self, content: &[u8]) -> Result<&mut Self> {
        self.content = content.to_vec();

        self.signature = self.read_signature()?;

        self.shader = super::RMLShader::from_air_file(
            air_parser::Parser::new(
                content[self.signature.bitcode_offset as usize
                    ..(self.signature.bitcode_offset + self.signature.bitcode_size) as usize]
                    .to_vec(),
            )?
            .start()?,
        );

        Ok(self)
    }

    pub fn read_signature(&mut self) -> Result<MTLLibrarySignature> {
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
            MTLLibraryPlatform::from_u16(u16::from_le_bytes([self.advance()?, self.advance()?]))?;

        let version = (
            u16::from_le_bytes([self.advance()?, self.advance()?]),
            u16::from_le_bytes([self.advance()?, self.advance()?]),
        );

        let library_type = MTLLibraryType::from_u8(self.advance()?)?;

        let target_os_type = MTLLibraryTargetOSType::from_u8(self.advance()?)?;

        let mut major = 0;
        let mut minor = 0;
        if !matches!(target_os_type, MTLLibraryTargetOSType::Unknown) {
            major = u16::from_le_bytes([self.advance()?, self.advance()?]);
            minor = u16::from_le_bytes([self.advance()?, self.advance()?]);
        }

        let target_os = MTLLibraryTargetOS::new(target_os_type, major, minor);

        let file_size = self.advance_u64()?;

        let function_list_offset = self.advance_u64()?;
        let function_list_size = self.advance_u64()?;

        let public_metadata_offset = self.advance_u64()?;
        let public_metadata_size = self.advance_u64()?;

        let private_metadata_offset = self.advance_u64()?;
        let private_metadata_size = self.advance_u64()?;

        let bitcode_offset = self.advance_u64()?;
        let bitcode_size = self.advance_u64()?;

        Ok(MTLLibrarySignature {
            target_platform,
            version,
            library_type,
            target_os,
            file_size,
            function_list_offset,
            function_list_size,
            public_metadata_offset,
            public_metadata_size,
            private_metadata_offset,
            private_metadata_size,
            bitcode_offset,
            bitcode_size,
        })
    }

    pub fn advance_u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes([
            self.advance()?,
            self.advance()?,
            self.advance()?,
            self.advance()?,
            self.advance()?,
            self.advance()?,
            self.advance()?,
            self.advance()?,
        ]))
    }

    pub fn jump_to(&mut self, value: usize) -> Result<()> {
        if value > self.content.len() {
            return Err(anyhow!("Invalid position to jump: out of bounds."));
        }

        Ok(self.position = value)
    }

    pub fn advance(&mut self) -> Result<u8> {
        self.position += 1;

        if self.position - 1 > self.content.len() {
            return Err(anyhow!("Position out of bounds."));
        }

        Ok(self.content[self.position - 1])
    }
}
