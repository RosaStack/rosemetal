pub mod items;

use anyhow::{Result, anyhow};
pub use items::*;

pub struct Parser {
    position: i64,
    signature: SpirVSignature,
    content: Vec<u8>,
    operands: Vec<SpirVOp>,
}

impl Parser {
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            position: -1,
            signature: SpirVSignature::default(),
            content,
            operands: vec![],
        }
    }

    pub fn advance(&mut self) -> Result<u8> {
        self.position += 1;

        if self.position as usize > self.content.len() {
            return Err(anyhow!("Position is bigger than the contents length."));
        }

        Ok(self.content[self.position as usize])
    }

    pub fn get_signature(&mut self) -> Result<SpirVSignature> {
        let magic_number = u32::from_le_bytes([
            self.advance()?,
            self.advance()?,
            self.advance()?,
            self.advance()?,
        ]);

        if magic_number != 0x7230203 {
            return Err(anyhow!("Invalid or corrupted magic number."));
        }

        let version_hex = u32::from_be_bytes([
            self.advance()?,
            self.advance()?,
            self.advance()?,
            self.advance()?,
        ]);

        let version = (version_hex.to_le_bytes()[1], version_hex.to_le_bytes()[2]);

        // TODO: Find a way to parse the tool that generated this.
        let generator_magic_number = u32::from_le_bytes([
            self.advance()?,
            self.advance()?,
            self.advance()?,
            self.advance()?,
        ]);

        let bound = u32::from_le_bytes([
            self.advance()?,
            self.advance()?,
            self.advance()?,
            self.advance()?,
        ]);

        let reserved_instruction_schema = u32::from_le_bytes([
            self.advance()?,
            self.advance()?,
            self.advance()?,
            self.advance()?,
        ]);

        Ok(SpirVSignature {
            magic_number,
            version,
            generator_magic_number,
            bound,
            reserved_instruction_schema,
        })
    }

    pub fn parse_op_capability(&mut self) -> Result<SpirVCapability> {
        let instruction = u32::from_le_bytes([
            self.advance()?,
            self.advance()?,
            self.advance()?,
            self.advance()?,
        ]);

        Ok(match instruction {
            1 => SpirVCapability::Shader,
            _ => todo!(),
        })
    }

    pub fn parse_literal(&mut self) -> Result<String> {
        let mut byte_count = 0;
        let mut result = String::new();

        loop {
            if byte_count == 4 {
                byte_count = 0;
            }

            let character = self.advance()? as char;

            if character == '\0' {
                while byte_count < 3 {
                    self.advance()?;
                    byte_count += 1;
                }

                break;
            }

            result.push(character);
            byte_count += 1;
        }

        Ok(result)
    }

    pub fn parse_memory_model(&mut self) -> Result<SpirVMemoryModel> {
        let instruction = u32::from_le_bytes([
            self.advance()?,
            self.advance()?,
            self.advance()?,
            self.advance()?,
        ]);

        Ok(match instruction {
            0 => SpirVMemoryModel::Simple,
            1 => SpirVMemoryModel::Glsl450,
            2 => SpirVMemoryModel::OpenCL,
            3 => SpirVMemoryModel::Vulkan,
            _ => return Err(anyhow!("Invalid Memory Model.")),
        })
    }

    pub fn parse_addressing_model(&mut self) -> Result<SpirVAddressingModel> {
        let instruction = u32::from_le_bytes([
            self.advance()?,
            self.advance()?,
            self.advance()?,
            self.advance()?,
        ]);

        Ok(match instruction {
            0 => SpirVAddressingModel::Logical,
            1 => SpirVAddressingModel::Physical32,
            2 => SpirVAddressingModel::Physical64,
            3 => SpirVAddressingModel::PhysicalStorageBuffer64,
            _ => return Err(anyhow!("Invalid Addressing Model.")),
        })
    }

    pub fn parse_op(&mut self) -> Result<SpirVOp> {
        let word_count = u16::from_le_bytes([self.advance()?, self.advance()?]);
        let op_code = u16::from_le_bytes([self.advance()?, self.advance()?]);

        let mut result = SpirVOp::default();

        result.value = match (op_code, word_count) {
            (2, 17) => SpirVValue::Capability(self.parse_op_capability()?),
            (6, 11) => {
                result.id = u32::from_le_bytes([
                    self.advance()?,
                    self.advance()?,
                    self.advance()?,
                    self.advance()?,
                ]);
                SpirVValue::ExtendedInstructionImport(self.parse_literal()?)
            }
            (3, 14) => {
                let addressing_model = self.parse_addressing_model()?;
                let memory_model = self.parse_memory_model()?;
                SpirVValue::MemoryModel(addressing_model, memory_model)
            }
            _ => todo!("{:?}", (op_code, word_count)),
        };

        Ok(result)
    }

    pub fn start(&mut self) -> Result<()> {
        self.signature = self.get_signature()?;
        let mut operands: Vec<SpirVOp> = vec![];

        let pos = self.position as usize;
        while pos < self.content.len() {
            operands.push(self.parse_op()?);
        }

        todo!("{:?}", self.signature);
    }
}
