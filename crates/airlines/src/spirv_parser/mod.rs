pub mod items;

use std::{
    collections::{BTreeMap, HashMap},
    u32,
};

use anyhow::{Result, anyhow};
pub use items::*;

#[derive(Default, Debug, Clone)]
pub struct SpirVModule {
    pub signature: SpirVSignature,
    pub operands: Vec<SpirVOp>,
    pub addressing_model: Option<SpirVAddressingModel>,
    pub memory_model: Option<SpirVMemoryModel>,
    pub name_table: HashMap<SpirVVariableId, SpirVName>,
    pub entry_point_table: HashMap<SpirVVariableId, SpirVEntryPoint>,
    pub decorate_table: HashMap<SpirVVariableId, SpirVDecorate>,
    pub type_table: BTreeMap<SpirVVariableId, SpirVType>,
    pub alloca_table: HashMap<SpirVVariableId, SpirVAlloca>,
    pub constants_table: HashMap<SpirVVariableId, SpirVConstant>,
    pub constant_composites_table: HashMap<SpirVVariableId, SpirVConstantComposite>,
    pub functions_table: HashMap<SpirVVariableId, SpirVFunction>,
    pub capabilities: Vec<SpirVCapability>,
}

pub struct Parser {
    pub position: i64,
    pub content: Vec<u32>,
    pub module: SpirVModule,
}

impl Parser {
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            position: -1,
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
            module: SpirVModule::default(),
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
                Self::add_capability_to(SpirVCapability::Shader, &mut self.module.capabilities)?;
                SpirVExecutionModel::Vertex
            }
            1 => {
                Self::add_capability_to(
                    SpirVCapability::Tessellation,
                    &mut self.module.capabilities,
                )?;
                SpirVExecutionModel::TessellationControl
            }
            2 => {
                Self::add_capability_to(
                    SpirVCapability::Tessellation,
                    &mut self.module.capabilities,
                )?;
                SpirVExecutionModel::TessellationEvaluation
            }
            3 => {
                Self::add_capability_to(SpirVCapability::Geometry, &mut self.module.capabilities)?;
                SpirVExecutionModel::Geometry
            }
            4 => {
                Self::add_capability_to(SpirVCapability::Shader, &mut self.module.capabilities)?;
                SpirVExecutionModel::Fragment
            }
            5 => {
                Self::add_capability_to(SpirVCapability::Shader, &mut self.module.capabilities)?;
                SpirVExecutionModel::GLCompute
            }
            6 => {
                Self::add_capability_to(SpirVCapability::Kernel, &mut self.module.capabilities)?;
                SpirVExecutionModel::Kernel
            }
            5267 => {
                Self::add_capability_to(
                    SpirVCapability::MeshShadingNV,
                    &mut self.module.capabilities,
                )?;
                SpirVExecutionModel::TaskNV
            }
            5268 => {
                Self::add_capability_to(
                    SpirVCapability::MeshShadingNV,
                    &mut self.module.capabilities,
                )?;
                SpirVExecutionModel::MeshNV
            }
            5313 => {
                Self::add_capability_to(
                    SpirVCapability::RayTracingKHR,
                    &mut self.module.capabilities,
                )?;
                SpirVExecutionModel::RayGenerationKHR
            }
            5314 => {
                Self::add_capability_to(
                    SpirVCapability::RayTracingKHR,
                    &mut self.module.capabilities,
                )?;
                SpirVExecutionModel::IntersectionKHR
            }
            5315 => {
                Self::add_capability_to(
                    SpirVCapability::RayTracingKHR,
                    &mut self.module.capabilities,
                )?;
                SpirVExecutionModel::AnyHitKHR
            }
            5316 => {
                Self::add_capability_to(
                    SpirVCapability::RayTracingKHR,
                    &mut self.module.capabilities,
                )?;
                SpirVExecutionModel::MissKHR
            }
            5317 => {
                Self::add_capability_to(
                    SpirVCapability::RayTracingKHR,
                    &mut self.module.capabilities,
                )?;
                SpirVExecutionModel::CallableKHR
            }
            5364 => {
                Self::add_capability_to(
                    SpirVCapability::MeshShadingEXT,
                    &mut self.module.capabilities,
                )?;
                SpirVExecutionModel::TaskEXT
            }
            5365 => {
                Self::add_capability_to(
                    SpirVCapability::MeshShadingEXT,
                    &mut self.module.capabilities,
                )?;
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

        Ok(match SpirVOpCode::from_u32(op_code as u32) {
            SpirVOpCode::Source => {
                let source_language = self.parse_source_language()?;
                // word_count - source_language
                let words_left = word_count - 3;
                let version = self.advance()?;

                if words_left != 0 {
                    todo!("Handle optional <id> File and <Literal> Source.");
                }

                SpirVOp::Source(SpirVSource {
                    source_language,
                    version,
                })
            }
            SpirVOpCode::SourceExtension => SpirVOp::SourceExtension(self.parse_literal()?.1),
            SpirVOpCode::Name => {
                let id = SpirVVariableId(self.advance()?);
                let name = self.parse_literal()?.1;
                self.module.name_table.insert(
                    id,
                    SpirVName {
                        name: name.clone(),
                        member_names: vec![],
                    },
                );
                SpirVOp::Name(id, name)
            }
            SpirVOpCode::MemberName => {
                let id = SpirVVariableId(self.advance()?);
                let member_id = self.advance()? as usize;
                let member_name = self.parse_literal()?.1;
                let member_names_vec =
                    &mut self.module.name_table.get_mut(&id).unwrap().member_names;

                if member_names_vec.len() <= member_id {
                    let difference = member_id - member_names_vec.len() + 1;
                    member_names_vec.resize(member_names_vec.len() + difference, String::new());
                }

                member_names_vec[member_id] = member_name.clone();
                SpirVOp::MemberName(id, member_id, member_name)
            }
            SpirVOpCode::ExtInstImport => {
                let result_id = SpirVVariableId(self.advance()?);
                SpirVOp::ExtendedInstructionImport(result_id, self.parse_literal()?.1)
            }
            SpirVOpCode::MemoryModel => {
                let addressing_model = self.parse_addressing_model()?;
                let memory_model = self.parse_memory_model()?;
                self.module.addressing_model = Some(addressing_model.clone());
                self.module.memory_model = Some(memory_model.clone());
                SpirVOp::MemoryModel(addressing_model, memory_model)
            }
            SpirVOpCode::EntryPoint => {
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

                let entry_point = SpirVEntryPoint {
                    name: name.1,
                    execution_model,
                    entry_point_id,
                    arguments,
                };

                self.module
                    .entry_point_table
                    .insert(entry_point_id, entry_point.clone());
                SpirVOp::EntryPoint(entry_point)
            }
            SpirVOpCode::Capability => {
                let capability = self.parse_op_capability()?;
                Self::add_capability_to(capability.clone(), &mut self.module.capabilities)?;
                Self::update_implicit_capabilities(&mut self.module.capabilities)?;
                SpirVOp::Capability(capability)
            }
            SpirVOpCode::TypeVoid => {
                let target_id = SpirVVariableId(self.advance()?);
                self.module.type_table.insert(target_id, SpirVType::Void);

                SpirVOp::Type(target_id, SpirVType::Void)
            }
            SpirVOpCode::TypeInt => {
                let target_id = SpirVVariableId(self.advance()?);
                let width = self.advance()?;
                let is_signed = self.advance()? != 0;

                self.module
                    .type_table
                    .insert(target_id, SpirVType::Int(width, is_signed));
                SpirVOp::Type(target_id, SpirVType::Int(width, is_signed))
            }
            SpirVOpCode::TypeFloat => {
                let words_left = word_count - 3;
                let target_id = SpirVVariableId(self.advance()?);
                let width = self.advance()?;

                if words_left != 0 {
                    todo!("Parsing the floating point encoding.");
                }

                self.module
                    .type_table
                    .insert(target_id, SpirVType::Float(width));
                SpirVOp::Type(target_id, SpirVType::Float(width))
            }
            SpirVOpCode::TypeVector => {
                let target_id = SpirVVariableId(self.advance()?);
                let vector_type = SpirVVariableId(self.advance()?);
                let size = self.advance()?;

                self.module
                    .type_table
                    .insert(target_id, SpirVType::Vector(vector_type, size));

                SpirVOp::Type(target_id, SpirVType::Vector(vector_type, size))
            }
            SpirVOpCode::TypeArray => {
                let target_id = SpirVVariableId(self.advance()?);
                let array_type = SpirVVariableId(self.advance()?);
                let length = SpirVVariableId(self.advance()?);

                self.module
                    .type_table
                    .insert(target_id, SpirVType::Array(array_type, length));

                SpirVOp::Type(target_id, SpirVType::Array(array_type, length))
            }
            SpirVOpCode::TypePointer => {
                let target_id = SpirVVariableId(self.advance()?);
                let storage_class = self.parse_storage_class()?;
                let type_id = SpirVVariableId(self.advance()?);

                self.module
                    .type_table
                    .insert(target_id, SpirVType::Pointer(storage_class, type_id));

                SpirVOp::Type(target_id, SpirVType::Pointer(storage_class, type_id))
            }
            SpirVOpCode::TypeFunction => {
                let target_id = SpirVVariableId(self.advance()?);
                let type_id = SpirVVariableId(self.advance()?);

                let mut args = vec![];
                for _i in 0..word_count - 3 {
                    args.push(SpirVVariableId(self.advance()?));
                }

                self.module
                    .type_table
                    .insert(target_id, SpirVType::Function(type_id, args.clone()));

                SpirVOp::Type(target_id, SpirVType::Function(type_id, args))
            }
            SpirVOpCode::Constant => {
                let type_id = SpirVVariableId(self.advance()?);
                let target_id = SpirVVariableId(self.advance()?);

                let constant = {
                    let words_left = word_count - 3;
                    let mut values = vec![];
                    for _i in 0..words_left {
                        values.push(self.advance()?.to_le_bytes());
                    }

                    match self.module.type_table.get(&type_id) {
                        Some(ty) => match ty {
                            SpirVType::Int(width, is_signed) => SpirVConstant {
                                type_id,
                                value: if *is_signed {
                                    SpirVConstantValue::SignedInteger(match width {
                                        8 => i8::from_le_bytes([values[0][0]]) as i64,
                                        16 => {
                                            i16::from_le_bytes([values[0][0], values[0][1]]) as i64
                                        }
                                        32 => i32::from_le_bytes([
                                            values[0][0],
                                            values[0][1],
                                            values[0][2],
                                            values[0][3],
                                        ]) as i64,
                                        64 => i64::from_le_bytes([
                                            values[0][0],
                                            values[0][1],
                                            values[0][2],
                                            values[0][3],
                                            values[1][0],
                                            values[1][1],
                                            values[1][2],
                                            values[1][3],
                                        ]),
                                        _ => {
                                            return Err(anyhow!(
                                                "Invalid Signed Integer Width '{:?}'",
                                                width
                                            ));
                                        }
                                    })
                                } else {
                                    SpirVConstantValue::UnsignedInteger(match width {
                                        8 => u8::from_le_bytes([values[0][0]]) as u64,
                                        16 => {
                                            u16::from_le_bytes([values[0][0], values[0][1]]) as u64
                                        }
                                        32 => u32::from_le_bytes([
                                            values[0][0],
                                            values[0][1],
                                            values[0][2],
                                            values[0][3],
                                        ]) as u64,
                                        64 => u64::from_le_bytes([
                                            values[0][0],
                                            values[0][1],
                                            values[0][2],
                                            values[0][3],
                                            values[1][0],
                                            values[1][1],
                                            values[1][2],
                                            values[1][3],
                                        ]),
                                        _ => {
                                            return Err(anyhow!(
                                                "Invalid Unsigned Integer Width '{:?}'",
                                                width
                                            ));
                                        }
                                    })
                                },
                            },
                            SpirVType::Float(width) => SpirVConstant {
                                type_id,
                                value: match width {
                                    32 => SpirVConstantValue::Float32(f32::from_le_bytes([
                                        values[0][0],
                                        values[0][1],
                                        values[0][2],
                                        values[0][3],
                                    ])),
                                    64 => SpirVConstantValue::Float64(f64::from_le_bytes([
                                        values[0][0],
                                        values[0][1],
                                        values[0][2],
                                        values[0][3],
                                        values[1][0],
                                        values[1][1],
                                        values[1][2],
                                        values[1][3],
                                    ])),
                                    _ => {
                                        return Err(anyhow!(
                                            "Expected 32 or 64 width, found {:?}",
                                            width
                                        ));
                                    }
                                },
                            },
                            _ => {
                                return Err(anyhow!(
                                    "Expected Integer or Floating-Point Types, found '{:?}'",
                                    ty
                                ));
                            }
                        },
                        None => {
                            return Err(anyhow!(
                                "Type ID {:?} not found in Type Table.",
                                type_id.0
                            ));
                        }
                    }
                };

                self.module
                    .constants_table
                    .insert(target_id, constant.clone());
                SpirVOp::Constant(target_id, constant)
            }
            SpirVOpCode::ConstantComposite => {
                let type_id = SpirVVariableId(self.advance()?);
                let target_id = SpirVVariableId(self.advance()?);

                let mut values = vec![];
                for _i in 0..word_count - 3 {
                    values.push(SpirVVariableId(self.advance()?));
                }

                let constant_composite = SpirVConstantComposite { type_id, values };

                self.module
                    .constant_composites_table
                    .insert(target_id, constant_composite.clone());
                SpirVOp::ConstantComposite(target_id, constant_composite)
            }
            SpirVOpCode::Variable => {
                let type_id = SpirVVariableId(self.advance()?);

                if !matches!(
                    self.module.type_table.get(&type_id).unwrap(),
                    SpirVType::Pointer(_, _)
                ) {
                    return Err(anyhow!("Result Type must've be a Pointer Type."));
                }

                let target_id = SpirVVariableId(self.advance()?);
                let storage_class = self.parse_storage_class()?;
                let initializer = match word_count - 4 == 0 {
                    false => Some(SpirVVariableId(self.advance()?)),
                    true => None,
                };

                let alloca = SpirVAlloca {
                    type_id,
                    storage_class,
                    initializer,
                };

                self.module.alloca_table.insert(target_id, alloca.clone());

                SpirVOp::Alloca(target_id, alloca)
            }
            SpirVOpCode::Decorate => {
                let target_id = SpirVVariableId(self.advance()?);
                let decorate = self.parse_decorate_type()?;
                self.module.decorate_table.insert(
                    target_id,
                    SpirVDecorate {
                        ty: decorate.clone(),
                        member_decorates: vec![],
                    },
                );

                SpirVOp::Decorate(target_id, decorate)
            }
            SpirVOpCode::MemberDecorate => {
                let struct_id = SpirVVariableId(self.advance()?);
                let member_id = self.advance()? as usize;
                let member_decorate = self.parse_decorate_type()?;
                let member_decorates_vec = &mut self
                    .module
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

                SpirVOp::MemberDecorate(struct_id, member_id, member_decorate)
            }
            SpirVOpCode::TypeStruct => {
                let target_id = SpirVVariableId(self.advance()?);

                let mut types = vec![];
                for _i in 0..word_count - 2 {
                    types.push(SpirVVariableId(self.advance()?));
                }

                self.module
                    .type_table
                    .insert(target_id, SpirVType::Struct(types.clone()));

                SpirVOp::Type(target_id, SpirVType::Struct(types))
            }
            SpirVOpCode::Function => {
                let return_type_id = SpirVVariableId(self.advance()?);
                let result_id = SpirVVariableId(self.advance()?);
                let function_control = FunctionControl::from_u32(self.advance()?);
                let function_type_id = SpirVVariableId(self.advance()?);

                let mut instructions: Vec<SpirVOp> = vec![];
                let mut op = self.parse_op()?;

                while !matches!(op, SpirVOp::FunctionEnd) {
                    instructions.push(op.clone());
                    op = self.parse_op()?;
                }

                instructions.push(op.clone());

                let function = SpirVFunction {
                    function_type_id,
                    return_type_id,
                    function_control,
                    instructions,
                };

                self.module
                    .functions_table
                    .insert(result_id, function.clone());

                SpirVOp::Function(result_id, function)
            }
            SpirVOpCode::Label => {
                let result_id = SpirVVariableId(self.advance()?);

                let mut instructions: Vec<SpirVOp> = vec![];
                let mut op = self.parse_op()?;

                while !matches!(op, SpirVOp::Return) {
                    instructions.push(op.clone());
                    op = self.parse_op()?;
                }

                instructions.push(op.clone());

                let block = SpirVBlock { instructions };

                SpirVOp::Block(result_id, block)
            }
            SpirVOpCode::Store => {
                let pointer_id = SpirVVariableId(self.advance()?);
                let object_id = SpirVVariableId(self.advance()?);
                let memory_operands = match word_count - 3 == 0 {
                    false => SpirVMemoryOperands::from_u32(self.advance()?),
                    true => SpirVMemoryOperands::None,
                };

                SpirVOp::Store(SpirVStore {
                    pointer_id,
                    object_id,
                    memory_operands,
                })
            }
            SpirVOpCode::Load => {
                let type_id = SpirVVariableId(self.advance()?);
                let result_id = SpirVVariableId(self.advance()?);
                let pointer_id = SpirVVariableId(self.advance()?);
                let memory_operands = match word_count - 4 == 0 {
                    false => SpirVMemoryOperands::from_u32(self.advance()?),
                    true => SpirVMemoryOperands::None,
                };

                SpirVOp::Load(
                    result_id,
                    SpirVLoad {
                        type_id,
                        pointer_id,
                        memory_operands,
                    },
                )
            }
            SpirVOpCode::AccessChain => {
                let type_id = SpirVVariableId(self.advance()?);
                let result_id = SpirVVariableId(self.advance()?);
                let base_id = SpirVVariableId(self.advance()?);

                let mut indices = vec![];
                for _i in 0..word_count - 4 {
                    indices.push(SpirVVariableId(self.advance()?));
                }

                SpirVOp::AccessChain(
                    result_id,
                    SpirVAccessChain {
                        type_id,
                        base_id,
                        indices,
                    },
                )
            }
            SpirVOpCode::CompositeExtract => {
                let type_id = SpirVVariableId(self.advance()?);
                let result_id = SpirVVariableId(self.advance()?);
                let composite_id = SpirVVariableId(self.advance()?);
                let mut indices = vec![];
                for _i in 0..word_count - 4 {
                    indices.push(self.advance()?);
                }

                SpirVOp::CompositeExtract(
                    result_id,
                    SpirVCompositeExtract {
                        type_id,
                        composite_id,
                        indices,
                    },
                )
            }
            SpirVOpCode::CompositeConstruct => {
                let type_id = SpirVVariableId(self.advance()?);
                let result_id = SpirVVariableId(self.advance()?);
                let mut elements = vec![];
                for _i in 0..word_count - 3 {
                    elements.push(SpirVVariableId(self.advance()?));
                }

                SpirVOp::CompositeConstruct(
                    result_id,
                    SpirVCompositeConstruct { type_id, elements },
                )
            }
            SpirVOpCode::Return => SpirVOp::Return,
            SpirVOpCode::FunctionEnd => SpirVOp::FunctionEnd,
            _ => todo!("{:?}", (op_code, word_count)),
        })
    }

    pub fn parse_storage_class(&mut self) -> Result<SpirVStorageClass> {
        let v = self.advance()?;
        Ok(match v {
            0 => SpirVStorageClass::UniformConstant,
            1 => SpirVStorageClass::Input,
            2 => SpirVStorageClass::Uniform,
            3 => SpirVStorageClass::Output,
            4 => SpirVStorageClass::Workgroup,
            5 => SpirVStorageClass::CrossWorkgroup,
            6 => SpirVStorageClass::Private,
            7 => SpirVStorageClass::Function,
            8 => SpirVStorageClass::Generic,
            9 => SpirVStorageClass::PushConstant,
            10 => SpirVStorageClass::AtomicCounter,
            11 => SpirVStorageClass::Image,
            12 => SpirVStorageClass::StorageBuffer,
            _ => todo!("{:?}", v),
        })
    }

    pub fn parse_built_in(&mut self) -> Result<SpirVBuiltIn> {
        let v = self.advance()?;
        Ok(match v {
            0 => {
                Self::add_capability_to(SpirVCapability::Shader, &mut self.module.capabilities)?;
                SpirVBuiltIn::Position
            }
            1 => {
                Self::add_capability_to(SpirVCapability::Shader, &mut self.module.capabilities)?;
                SpirVBuiltIn::PointSize
            }
            3 => {
                Self::add_capability_to(
                    SpirVCapability::ClipDistance,
                    &mut self.module.capabilities,
                )?;
                SpirVBuiltIn::ClipDistance
            }
            4 => {
                Self::add_capability_to(
                    SpirVCapability::CullDistance,
                    &mut self.module.capabilities,
                )?;
                SpirVBuiltIn::CullDistance
            }
            42 => {
                Self::add_capability_to(SpirVCapability::Shader, &mut self.module.capabilities)?;
                SpirVBuiltIn::VertexIndex
            }
            _ => todo!(),
        })
    }

    pub fn parse_decorate_type(&mut self) -> Result<SpirVDecorateType> {
        let v = self.advance()?;

        Ok(match v {
            2 => {
                Self::add_capability_to(SpirVCapability::Shader, &mut self.module.capabilities)?;
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

    pub fn start(&mut self) -> Result<SpirVModule> {
        self.module.signature = self.get_signature()?;

        let mut pos = self.position as usize;
        while pos < self.content.len() - 1 {
            let op = self.parse_op()?;
            self.module.operands.push(op);
            pos = self.position as usize;
        }

        Ok(self.module.clone())
    }
}
