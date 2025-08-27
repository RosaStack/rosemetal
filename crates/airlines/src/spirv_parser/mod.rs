pub mod items;

use std::u32;

use anyhow::{Result, anyhow};
pub use items::*;

pub struct Parser {
    position: i64,
    signature: SpirVSignature,
    content: Vec<u8>,
    operands: Vec<SpirVOp>,
    capabilities: Vec<SpirVCapability>,
}

impl Parser {
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            position: -1,
            signature: SpirVSignature::default(),
            content,
            operands: vec![],
            capabilities: vec![],
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
            0 => SpirVCapability::Matrix,
            1 => SpirVCapability::Shader,
            2 => SpirVCapability::Geometry,
            3 => SpirVCapability::Tessellation,
            4 => SpirVCapability::Addresses,
            5 => SpirVCapability::Linkage,
            6 => SpirVCapability::Kernel,
            7 => SpirVCapability::Vector16,
            8 => SpirVCapability::Float16Buffer,
            9 => SpirVCapability::Float16,
            10 => SpirVCapability::Float64,
            11 => SpirVCapability::Int64,
            12 => SpirVCapability::Int64Atomics,
            13 => SpirVCapability::ImageBasic,
            14 => SpirVCapability::ImageReadWrite,
            15 => SpirVCapability::ImageMipmap,
            17 => SpirVCapability::Pipes,
            18 => SpirVCapability::Groups,
            19 => SpirVCapability::DeviceEnqueue,
            20 => SpirVCapability::LiteralSampler,
            21 => SpirVCapability::AtomicStorage,
            22 => SpirVCapability::Int16,
            23 => SpirVCapability::TessellationPointSize,
            24 => SpirVCapability::GeometryPointSize,
            25 => SpirVCapability::ImageGatherExtended,
            27 => SpirVCapability::StorageImageMultisample,
            28 => SpirVCapability::UniformBufferArrayDynamicIndexing,
            29 => SpirVCapability::SampledImageArrayDynamicIndexing,
            30 => SpirVCapability::StorageBufferArrayDynamicIndexing,
            31 => SpirVCapability::StorageImageArrayDynamicIndexing,
            32 => SpirVCapability::ClipDistance,
            33 => SpirVCapability::CullDistance,
            34 => SpirVCapability::ImageCubeArray,
            35 => SpirVCapability::SampleRateShading,
            36 => SpirVCapability::ImageRect,
            37 => SpirVCapability::SampledRect,
            38 => SpirVCapability::GenericPointer,
            39 => SpirVCapability::Int8,
            40 => SpirVCapability::InputAttachment,
            41 => SpirVCapability::SparseResidency,
            42 => SpirVCapability::MinLod,
            43 => SpirVCapability::Sampled1D,
            44 => SpirVCapability::Image1D,
            45 => SpirVCapability::SampledCubeArray,
            46 => SpirVCapability::SampledBuffer,
            47 => SpirVCapability::ImageBuffer,
            48 => SpirVCapability::ImageMSArray,
            49 => SpirVCapability::StorageImageExtendedFormats,
            50 => SpirVCapability::ImageQuery,
            51 => SpirVCapability::DerivativeControl,
            52 => SpirVCapability::InterpolationFunction,
            53 => SpirVCapability::TransformFeedback,
            54 => SpirVCapability::GeometryStreams,
            55 => SpirVCapability::StorageImageReadWithoutFormat,
            56 => SpirVCapability::StorageImageWriteWithoutFormat,
            57 => SpirVCapability::MultiViewport,
            58.. => todo!("Capabilities from 1.1 and above and Vendor Specific."),
            _ => return Err(anyhow!("Invalid Capability ID.")),
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

    pub fn add_capability_to(
        capability: SpirVCapability,
        capabilities: &mut Vec<SpirVCapability>,
    ) -> Result<()> {
        let mut is_value_new = true;
        for i in &mut *capabilities {
            if *i == capability {
                is_value_new = false;
            }
        }

        if is_value_new {
            capabilities.push(capability);
        }

        Ok(())
    }

    pub fn update_implicit_capabilities(capabilities: &mut Vec<SpirVCapability>) -> Result<()> {
        // ========================================================
        // TODO: Add version and vendor specific implicit declares.
        // ========================================================

        let mut push_capabilities: Vec<SpirVCapability> = vec![];
        for i in &mut *capabilities {
            match i {
                SpirVCapability::Shader => {
                    Self::add_capability_to(SpirVCapability::Matrix, &mut push_capabilities)?
                }
                SpirVCapability::Geometry
                | SpirVCapability::Tessellation
                | SpirVCapability::AtomicStorage
                | SpirVCapability::ImageGatherExtended
                | SpirVCapability::StorageImageMultisample
                | SpirVCapability::UniformBufferArrayDynamicIndexing
                | SpirVCapability::SampledImageArrayDynamicIndexing
                | SpirVCapability::StorageBufferArrayDynamicIndexing
                | SpirVCapability::StorageImageArrayDynamicIndexing
                | SpirVCapability::ClipDistance
                | SpirVCapability::CullDistance
                | SpirVCapability::SampleRateShading
                | SpirVCapability::SampledRect
                | SpirVCapability::InputAttachment
                | SpirVCapability::SparseResidency
                | SpirVCapability::MinLod
                | SpirVCapability::SampledCubeArray
                | SpirVCapability::ImageMSArray
                | SpirVCapability::StorageImageExtendedFormats
                | SpirVCapability::ImageQuery
                | SpirVCapability::DerivativeControl
                | SpirVCapability::InterpolationFunction
                | SpirVCapability::TransformFeedback
                | SpirVCapability::StorageImageReadWithoutFormat
                | SpirVCapability::StorageImageWriteWithoutFormat => {
                    Self::add_capability_to(SpirVCapability::Shader, &mut push_capabilities)?
                }
                SpirVCapability::Vector16
                | SpirVCapability::Float16Buffer
                | SpirVCapability::ImageBasic
                | SpirVCapability::Pipes
                | SpirVCapability::DeviceEnqueue
                | SpirVCapability::LiteralSampler => {
                    Self::add_capability_to(SpirVCapability::Kernel, &mut push_capabilities)?
                }
                SpirVCapability::Int64Atomics => {
                    Self::add_capability_to(SpirVCapability::Int64, &mut push_capabilities)?
                }
                SpirVCapability::ImageReadWrite | SpirVCapability::ImageMipmap => {
                    Self::add_capability_to(SpirVCapability::ImageBasic, &mut push_capabilities)?
                }
                SpirVCapability::TessellationPointSize => {
                    Self::add_capability_to(SpirVCapability::Tessellation, &mut push_capabilities)?
                }
                SpirVCapability::GeometryPointSize
                | SpirVCapability::GeometryStreams
                | SpirVCapability::MultiViewport => {
                    Self::add_capability_to(SpirVCapability::Geometry, &mut push_capabilities)?
                }
                SpirVCapability::ImageCubeArray => Self::add_capability_to(
                    SpirVCapability::SampledCubeArray,
                    &mut push_capabilities,
                )?,
                SpirVCapability::ImageRect => {
                    Self::add_capability_to(SpirVCapability::SampledRect, &mut push_capabilities)?
                }
                SpirVCapability::GenericPointer => {
                    Self::add_capability_to(SpirVCapability::Addresses, &mut push_capabilities)?
                }
                SpirVCapability::Image1D => {
                    Self::add_capability_to(SpirVCapability::Sampled1D, &mut push_capabilities)?
                }
                SpirVCapability::ImageBuffer => {
                    Self::add_capability_to(SpirVCapability::SampledBuffer, &mut push_capabilities)?
                }
                _ => {}
            }
        }

        if push_capabilities.len() != 0 {
            Self::update_implicit_capabilities(&mut push_capabilities)?;
        }

        for i in push_capabilities {
            Self::add_capability_to(i, capabilities)?;
        }

        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        self.signature = self.get_signature()?;
        let mut operands: Vec<SpirVOp> = vec![];

        let pos = self.position as usize;
        while pos < self.content.len() {
            let op = self.parse_op()?;

            match &op.value {
                SpirVValue::Capability(capability) => {
                    Self::add_capability_to(capability.clone(), &mut self.capabilities)?;
                    Self::update_implicit_capabilities(&mut self.capabilities)?;
                }
                _ => {}
            }

            operands.push(op);
        }

        todo!("{:?}", self.signature);
    }
}
