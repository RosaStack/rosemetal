#[derive(Debug, Default, Clone)]
pub struct SpirVSignature {
    pub magic_number: u32,
    pub version: (u8, u8),
    pub generator_magic_number: u32,
    pub bound: u32,
    pub reserved_instruction_schema: u32,
}

#[derive(Debug, Default, Clone)]
pub struct SpirVOp {
    pub id: u32,
    pub value: SpirVValue,
}

#[derive(Debug, Default, Clone)]
pub enum SpirVValue {
    #[default]
    Empty,
    Capability(SpirVCapability),
    ExtendedInstructionImport(String),
    MemoryModel(SpirVAddressingModel, SpirVMemoryModel),
}

#[derive(Debug, Default, Clone)]
pub enum SpirVAddressingModel {
    #[default]
    Logical,
    Physical32,
    Physical64,
    PhysicalStorageBuffer64,
}

#[derive(Debug, Default, Clone)]
pub enum SpirVMemoryModel {
    #[default]
    Simple,
    Glsl450,
    OpenCL,
    Vulkan,
}

#[derive(Debug, Default, Clone)]
pub enum SpirVCapability {
    #[default]
    Shader,
}
