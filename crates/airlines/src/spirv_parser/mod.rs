pub mod items;

use std::{collections::HashMap, u32};

use anyhow::{Result, anyhow};
pub use items::*;

pub struct Parser {
    position: i64,
    signature: SpirVSignature,
    content: Vec<u32>,
    operands: Vec<SpirVOp>,
    name_table: HashMap<SpirVVariableId, SpirVName>,
    decorate_table: HashMap<SpirVVariableId, SpirVDecorate>,
    capabilities: Vec<SpirVCapability>,
}

impl Parser {
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            position: -1,
            signature: SpirVSignature::default(),
            content: {
                let mut result: Vec<u32> = vec![0; content.len() / 4];
                let mut count = 0;
                for i in &mut result {
                    *i = u32::from_le_bytes([
                        content[count * 4 + 0],
                        content[count * 4 + 1],
                        content[count * 4 + 2],
                        content[count * 4 + 3],
                    ]);
                    count += 1;
                }
                result
            },
            operands: vec![],
            capabilities: vec![],
            name_table: HashMap::new(),
            decorate_table: HashMap::new(),
        }
    }

    pub fn advance(&mut self) -> Result<u32> {
        self.position += 1;

        if self.position as usize >= self.content.len() {
            return Err(anyhow!("Position is bigger than the contents length."));
        }

        Ok(self.content[self.position as usize])
    }

    pub fn get_signature(&mut self) -> Result<SpirVSignature> {
        let magic_number = self.advance()?;

        if magic_number != 0x7230203 {
            return Err(anyhow!("Invalid or corrupted magic number."));
        }

        let version_hex = self.advance()?;

        let version = (version_hex.to_le_bytes()[1], version_hex.to_le_bytes()[2]);

        // TODO: Find a way to parse the tool that generated this.
        let generator_magic_number = self.advance()?;

        let bound = self.advance()?;

        let reserved_instruction_schema = self.advance()?;

        Ok(SpirVSignature {
            magic_number,
            version,
            generator_magic_number,
            bound,
            reserved_instruction_schema,
        })
    }

    pub fn parse_op_capability(&mut self) -> Result<SpirVCapability> {
        let instruction = self.advance()?;

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

    pub fn parse_literal(&mut self) -> Result<(u64, String)> {
        let mut words = 0;
        let mut result = String::new();

        'main_loop: loop {
            words += 1;
            let characters = self.advance()?.to_le_bytes();

            for i in characters {
                if i == 0 {
                    break 'main_loop;
                }

                result.push(i as char);
            }
        }

        Ok((words, result))
    }

    pub fn parse_memory_model(&mut self) -> Result<SpirVMemoryModel> {
        let instruction = self.advance()?;

        Ok(match instruction {
            0 => SpirVMemoryModel::Simple,
            1 => SpirVMemoryModel::Glsl450,
            2 => SpirVMemoryModel::OpenCL,
            3 => SpirVMemoryModel::Vulkan,
            _ => return Err(anyhow!("Invalid Memory Model.")),
        })
    }

    pub fn parse_addressing_model(&mut self) -> Result<SpirVAddressingModel> {
        let instruction = self.advance()?;

        Ok(match instruction {
            0 => SpirVAddressingModel::Logical,
            1 => SpirVAddressingModel::Physical32,
            2 => SpirVAddressingModel::Physical64,
            5348 => SpirVAddressingModel::PhysicalStorageBuffer64,
            _ => return Err(anyhow!("Invalid Addressing Model.")),
        })
    }

    pub fn parse_execution_model(&mut self) -> Result<SpirVExecutionModel> {
        let instruction = self.advance()?;

        Ok(match instruction {
            0 => {
                Self::add_capability_to(SpirVCapability::Shader, &mut self.capabilities)?;
                SpirVExecutionModel::Vertex
            }
            1 => {
                Self::add_capability_to(SpirVCapability::Tessellation, &mut self.capabilities)?;
                SpirVExecutionModel::TessellationControl
            }
            2 => {
                Self::add_capability_to(SpirVCapability::Tessellation, &mut self.capabilities)?;
                SpirVExecutionModel::TessellationEvaluation
            }
            3 => {
                Self::add_capability_to(SpirVCapability::Geometry, &mut self.capabilities)?;
                SpirVExecutionModel::Geometry
            }
            4 => {
                Self::add_capability_to(SpirVCapability::Shader, &mut self.capabilities)?;
                SpirVExecutionModel::Fragment
            }
            5 => {
                Self::add_capability_to(SpirVCapability::Shader, &mut self.capabilities)?;
                SpirVExecutionModel::GLCompute
            }
            6 => {
                Self::add_capability_to(SpirVCapability::Kernel, &mut self.capabilities)?;
                SpirVExecutionModel::Kernel
            }
            5267 => {
                Self::add_capability_to(SpirVCapability::MeshShadingNV, &mut self.capabilities)?;
                SpirVExecutionModel::TaskNV
            }
            5268 => {
                Self::add_capability_to(SpirVCapability::MeshShadingNV, &mut self.capabilities)?;
                SpirVExecutionModel::MeshNV
            }
            5313 => {
                Self::add_capability_to(SpirVCapability::RayTracingKHR, &mut self.capabilities)?;
                SpirVExecutionModel::RayGenerationKHR
            }
            5314 => {
                Self::add_capability_to(SpirVCapability::RayTracingKHR, &mut self.capabilities)?;
                SpirVExecutionModel::IntersectionKHR
            }
            5315 => {
                Self::add_capability_to(SpirVCapability::RayTracingKHR, &mut self.capabilities)?;
                SpirVExecutionModel::AnyHitKHR
            }
            5316 => {
                Self::add_capability_to(SpirVCapability::RayTracingKHR, &mut self.capabilities)?;
                SpirVExecutionModel::MissKHR
            }
            5317 => {
                Self::add_capability_to(SpirVCapability::RayTracingKHR, &mut self.capabilities)?;
                SpirVExecutionModel::CallableKHR
            }
            5364 => {
                Self::add_capability_to(SpirVCapability::MeshShadingEXT, &mut self.capabilities)?;
                SpirVExecutionModel::TaskEXT
            }
            5365 => {
                Self::add_capability_to(SpirVCapability::MeshShadingEXT, &mut self.capabilities)?;
                SpirVExecutionModel::MeshEXT
            }
            _ => return Err(anyhow!("Invalid Execution Model ID.")),
        })
    }

    pub fn parse_source_language(&mut self) -> Result<SpirVSourceLanguage> {
        let instruction = self.advance()?;

        Ok(match instruction {
            1 => SpirVSourceLanguage::Essl,
            2 => SpirVSourceLanguage::Glsl,
            3 => SpirVSourceLanguage::OpenCLC,
            4 => SpirVSourceLanguage::OpenCLCpp,
            5 => SpirVSourceLanguage::Hlsl,
            6 => SpirVSourceLanguage::CppForOpenCL,
            7 => SpirVSourceLanguage::Sycl,
            8 => SpirVSourceLanguage::HeroC,
            9 => SpirVSourceLanguage::Nzsl,
            10 => SpirVSourceLanguage::Wgsl,
            11 => SpirVSourceLanguage::Slang,
            12 => SpirVSourceLanguage::Zig,
            13 => SpirVSourceLanguage::Rust,
            0 | 14.. => SpirVSourceLanguage::Unknown,
        })
    }

    pub fn parse_op(&mut self) -> Result<SpirVOp> {
        let first_word = self.advance()?.to_le_bytes();
        let op_code = u16::from_le_bytes([first_word[0], first_word[1]]);
        let word_count = u16::from_le_bytes([first_word[2], first_word[3]]);

        let mut result = SpirVOp::default();

        result.value = match op_code {
            3 => {
                let source_language = self.parse_source_language()?;
                // word_count - source_language
                let words_left = word_count - 3;
                let version = self.advance()?;

                if words_left != 0 {
                    todo!("Handle optional <id> File and <Literal> Source.");
                }

                SpirVValue::Source(SpirVSource {
                    source_language,
                    version,
                })
            }
            4 => SpirVValue::SourceExtension(self.parse_literal()?.1),
            5 => {
                let id = SpirVVariableId(self.advance()?);
                let name = self.parse_literal()?.1;
                self.name_table.insert(
                    id,
                    SpirVName {
                        name: name.clone(),
                        member_names: vec![],
                    },
                );
                SpirVValue::Name(id, name)
            }
            6 => {
                let id = SpirVVariableId(self.advance()?);
                let member_id = self.advance()? as usize;
                let member_name = self.parse_literal()?.1;
                let member_names_vec = &mut self.name_table.get_mut(&id).unwrap().member_names;

                if member_names_vec.len() <= member_id {
                    let difference = member_id - member_names_vec.len() + 1;
                    member_names_vec.resize(member_names_vec.len() + difference, String::new());
                }

                member_names_vec[member_id] = member_name.clone();
                SpirVValue::MemberName(id, member_id, member_name)
            }
            11 => {
                result.id = self.advance()?;
                SpirVValue::ExtendedInstructionImport(self.parse_literal()?.1)
            }
            17 => {
                let capability = self.parse_op_capability()?;
                Self::add_capability_to(capability.clone(), &mut self.capabilities)?;
                Self::update_implicit_capabilities(&mut self.capabilities)?;
                SpirVValue::Capability(capability)
            }
            14 => {
                let addressing_model = self.parse_addressing_model()?;
                let memory_model = self.parse_memory_model()?;
                SpirVValue::MemoryModel(addressing_model, memory_model)
            }
            15 => {
                let execution_model = self.parse_execution_model()?;
                let entry_point_id = SpirVVariableId(self.advance()?);

                // word_count - first_word - execution_model - entry_point_id
                let mut words_left = word_count - 3;

                let name = self.parse_literal()?;
                words_left -= name.0 as u16;

                let mut arguments = vec![];
                for _i in 0..words_left {
                    arguments.push(SpirVVariableId(self.advance()?));
                }

                SpirVValue::EntryPoint(SpirVEntryPoint {
                    name: name.1,
                    execution_model,
                    entry_point_id,
                    arguments,
                })
            }
            19 => {
                todo!()
            }
            71 => {
                let target_id = SpirVVariableId(self.advance()?);
                let decorate = self.parse_decorate_type()?;
                self.decorate_table.insert(
                    target_id,
                    SpirVDecorate {
                        ty: decorate.clone(),
                        member_decorates: vec![],
                    },
                );

                SpirVValue::Decorate(target_id, decorate)
            }
            72 => {
                let struct_id = SpirVVariableId(self.advance()?);
                let member_id = self.advance()? as usize;
                let member_decorate = self.parse_decorate_type()?;
                let member_decorates_vec = &mut self
                    .decorate_table
                    .get_mut(&struct_id)
                    .unwrap()
                    .member_decorates;

                if member_decorates_vec.len() <= member_id {
                    let difference = member_id - member_decorates_vec.len() + 1;
                    member_decorates_vec.resize(
                        member_decorates_vec.len() + difference,
                        SpirVDecorateType::default(),
                    );
                }

                member_decorates_vec[member_id] = member_decorate.clone();

                SpirVValue::MemberDecorate(struct_id, member_id, member_decorate)
            }
            _ => todo!("{:?}", (op_code, word_count)),
        };

        Ok(result)
    }

    pub fn parse_built_in(&mut self) -> Result<SpirVBuiltIn> {
        let v = self.advance()?;
        Ok(match v {
            0 => {
                Self::add_capability_to(SpirVCapability::Shader, &mut self.capabilities)?;
                SpirVBuiltIn::Position
            }
            1 => {
                Self::add_capability_to(SpirVCapability::Shader, &mut self.capabilities)?;
                SpirVBuiltIn::PointSize
            }
            3 => {
                Self::add_capability_to(SpirVCapability::ClipDistance, &mut self.capabilities)?;
                SpirVBuiltIn::ClipDistance
            }
            4 => {
                Self::add_capability_to(SpirVCapability::CullDistance, &mut self.capabilities)?;
                SpirVBuiltIn::CullDistance
            }
            42 => {
                Self::add_capability_to(SpirVCapability::Shader, &mut self.capabilities)?;
                SpirVBuiltIn::VertexIndex
            }
            _ => todo!(),
        })
    }

    pub fn parse_decorate_type(&mut self) -> Result<SpirVDecorateType> {
        let v = self.advance()?;

        Ok(match v {
            2 => {
                Self::add_capability_to(SpirVCapability::Shader, &mut self.capabilities)?;
                SpirVDecorateType::Block
            }
            11 => SpirVDecorateType::BuiltIn(self.parse_built_in()?),
            30 => SpirVDecorateType::Location(self.advance()?),
            _ => todo!(),
        })
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
            operands.push(op);
        }

        todo!("{:?}", self.signature);
    }
}
