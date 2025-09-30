#[derive(Debug, Default, Clone)]
pub struct SpirVSignature {
    pub magic_number: u32,
    pub version: (u8, u8),
    pub generator_magic_number: u32,
    pub bound: u32,
    pub reserved_instruction_schema: u32,
}

#[derive(Debug, Default, Clone)]
pub enum SpirVOp {
    #[default]
    Empty,
    Capability(SpirVCapability),
    ExtendedInstructionImport(SpirVVariableId, String),
    MemoryModel(SpirVAddressingModel, SpirVMemoryModel),
    EntryPoint(SpirVEntryPoint),
    Source(SpirVSource),
    SourceExtension(String),
    Name(SpirVVariableId, String),
    MemberName(SpirVVariableId, usize, String),
    Decorate(SpirVVariableId, SpirVDecorateType),
    MemberDecorate(SpirVVariableId, usize, SpirVDecorateType),
    Type(SpirVVariableId, SpirVType),
    Constant(SpirVVariableId, SpirVConstant),
    ConstantComposite(SpirVVariableId, SpirVConstantComposite),
    Alloca(SpirVVariableId, SpirVAlloca),
    FunctionEnd,
    Block(SpirVVariableId, SpirVBlock),
    Store(SpirVStore),
    Load(SpirVVariableId, SpirVLoad),
    AccessChain(SpirVVariableId, SpirVAccessChain),
    CompositeExtract(SpirVVariableId, SpirVCompositeExtract),
    CompositeInsert(SpirVVariableId, SpirVCompositeInsert),
    CompositeConstruct(SpirVVariableId, SpirVCompositeConstruct),
    Return,
    ReturnValue(SpirVVariableId),
    Function(SpirVVariableId, SpirVFunction),
    BitCast(SpirVVariableId, SpirVBitCast),
    VectorShuffle(SpirVVariableId, SpirVVectorShuffle),
}

#[derive(Debug, Default, Clone)]
pub struct SpirVBitCast {
    pub variable: SpirVVariableId,
    pub to_type: SpirVVariableId,
}

#[derive(Debug, Default, Clone)]
pub struct SpirVCompositeConstruct {
    pub type_id: SpirVVariableId,
    pub elements: Vec<SpirVVariableId>,
}

#[derive(Debug, Default, Clone)]
pub struct SpirVCompositeExtract {
    pub type_id: SpirVVariableId,
    pub composite_id: SpirVVariableId,
    pub indices: Vec<u32>,
}

#[derive(Debug, Default, Clone)]
pub struct SpirVCompositeInsert {
    pub type_id: SpirVVariableId,
    pub object_id: SpirVVariableId,
    pub composite_id: SpirVVariableId,
    pub indices: Vec<u32>,
}

#[derive(Debug, Default, Clone)]
pub struct SpirVAccessChain {
    pub type_id: SpirVVariableId,
    pub base_id: SpirVVariableId,
    pub indices: Vec<SpirVVariableId>,
}

#[derive(Debug, Default, Clone)]
pub struct SpirVStore {
    pub pointer_id: SpirVVariableId,
    pub object_id: SpirVVariableId,
    pub memory_operands: SpirVMemoryOperands,
}

#[derive(Debug, Default, Clone)]
pub struct SpirVLoad {
    pub type_id: SpirVVariableId,
    pub pointer_id: SpirVVariableId,
    pub memory_operands: SpirVMemoryOperands,
}

#[derive(Debug, Default, Clone)]
#[repr(u32)]
pub enum SpirVMemoryOperands {
    #[default]
    None = 0x0,
    Volatile = 0x1,
    Aligned = 0x2,
    NonTemporal = 0x4,
    MakePointerAvailable = 0x8,
    MakePointerVisible = 0x10,
    NonPrivatePointer = 0x20,
    AliasScopeINTELMask = 0x10000,
    NoAliasINTELMask = 0x20000,
}

impl SpirVMemoryOperands {
    pub fn from_u32(v: u32) -> Self {
        match v {
            0x0 => Self::None,
            0x1 => Self::Volatile,
            0x2 => Self::Aligned,
            0x4 => Self::NonTemporal,
            0x8 => Self::MakePointerAvailable,
            0x10 => Self::MakePointerVisible,
            0x20 => Self::NonPrivatePointer,
            0x10000 => Self::AliasScopeINTELMask,
            0x20000 => Self::NoAliasINTELMask,
            _ => unimplemented!("{:#X}", v),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SpirVVectorShuffle {
    pub vec_type: SpirVVariableId,
    pub vec1: SpirVVariableId,
    pub vec2: SpirVVariableId,
    pub mask: Vec<u32>,
}

#[derive(Debug, Default, Clone)]
pub struct SpirVFunction {
    pub function_type_id: SpirVVariableId,
    pub return_type_id: SpirVVariableId,
    pub function_control: FunctionControl,
    pub instructions: Vec<SpirVOp>,
}

#[derive(Debug, Default, Clone)]
pub struct SpirVBlock {
    pub instructions: Vec<SpirVOp>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum SpirVType {
    #[default]
    Void,
    Function(SpirVVariableId, Vec<SpirVVariableId>),
    Float(u32),
    Int(u32, bool),
    Vector(SpirVVariableId, u32),
    Array(SpirVVariableId, u32),
    Pointer(SpirVStorageClass, SpirVVariableId),
    Struct(Vec<SpirVVariableId>),
}

#[derive(Debug, Default, Clone)]
pub struct SpirVAlloca {
    pub type_id: SpirVVariableId,
    pub storage_class: SpirVStorageClass,
    pub initializer: Option<SpirVVariableId>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SpirVStorageClass {
    #[default]
    UniformConstant,
    Input,
    Uniform,
    Output,
    Workgroup,
    CrossWorkgroup,
    Private,
    Function,
    Generic,
    PushConstant,
    AtomicCounter,
    Image,
    StorageBuffer,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SpirVConstant {
    pub type_id: SpirVVariableId,
    pub value: SpirVConstantValue,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SpirVConstantComposite {
    pub type_id: SpirVVariableId,
    pub values: Vec<SpirVVariableId>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum SpirVConstantValue {
    #[default]
    Undefined,
    SignedInteger(i64),
    UnsignedInteger(u64),
    Float32(f32),
    Float64(f64),
    Null,
}

#[derive(Debug, Default, Clone)]
pub struct SpirVDecorate {
    pub ty: SpirVDecorateType,
    pub member_decorates: Vec<SpirVDecorateType>,
}

#[derive(Debug, Default, Clone)]
pub enum SpirVDecorateType {
    #[default]
    Block,
    BuiltIn(SpirVBuiltIn),
    Location(u32),
}

#[derive(Debug, Default, Clone)]
pub enum SpirVBuiltIn {
    #[default]
    Position,
    PointSize,
    ClipDistance,
    CullDistance,
    VertexId,
    InstanceId,
    PrimitiveId,
    InvocationId,
    Layer,
    ViewportIndex,
    TessLevelOuter,
    TessLevelInner,
    TessCoord,
    PatchVertices,
    FragCoord,
    PointCoord,
    FrontFacing,
    SampleId,
    SamplePosition,
    SampleMask,
    FragDepth,
    HelperInvocation,
    NumWorkgroups,
    WorkgroupSize,
    WorkgroupId,
    LocalInvocationId,
    GlobalInvocationId,
    LocalInvocationIndex,
    WorkDim,
    GlobalSize,
    EnqueuedWorkgroupSize,
    GlobalOffset,
    GlobalLinearId,
    SubgroupSize,
    SubgroupMaxSize,
    NumSubgroups,
    NumEnqueuedSubgroups,
    SubgroupId,
    SubgroupLocalInvocationId,
    VertexIndex,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SpirVName {
    pub name: String,
    pub member_names: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct SpirVSource {
    pub source_language: SpirVSourceLanguage,
    pub version: u32,
}

#[derive(Debug, Default, Clone)]
pub struct SpirVEntryPoint {
    pub name: String,
    pub execution_model: SpirVExecutionModel,
    pub entry_point_id: SpirVVariableId,
    pub arguments: Vec<SpirVVariableId>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpirVVariableId(pub u32);

#[derive(Debug, Default, Clone)]
pub enum SpirVSourceLanguage {
    #[default]
    Unknown,
    Essl,
    Glsl,
    OpenCLC,
    OpenCLCpp,
    Hlsl,
    CppForOpenCL,
    Sycl,
    HeroC,
    Nzsl,
    Wgsl,
    Slang,
    Zig,
    Rust,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SpirVAddressingModel {
    #[default]
    Logical,
    Physical32,
    Physical64,
    PhysicalStorageBuffer64 = 5348,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SpirVMemoryModel {
    #[default]
    Simple,
    Glsl450,
    OpenCL,
    Vulkan,
}

#[derive(Debug, Default, Clone)]
pub enum SpirVExecutionModel {
    #[default]
    Vertex,
    TessellationControl,
    TessellationEvaluation,
    Geometry,
    Fragment,
    GLCompute,
    Kernel,
    TaskNV,
    MeshNV,
    RayGenerationKHR,
    IntersectionKHR,
    AnyHitKHR,
    ClosestHitKHR,
    MissKHR,
    CallableKHR,
    TaskEXT,
    MeshEXT,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum SpirVOpCode {
    #[default]
    Empty = 0,
    SourceLanguage = 3,
    SourceExtension = 4,
    Name = 5,
    MemberName = 6,
    ExtInstImport = 11,
    MemoryModel = 14,
    EntryPoint = 15,
    Capability = 17,
    TypeVoid = 19,
    TypeInt = 21,
    TypeFloat = 22,
    TypeVector = 23,
    TypeArray = 28,
    TypeStruct = 30,
    TypePointer = 32,
    TypeFunction = 33,
    Constant = 43,
    ConstantComposite = 44,
    Function = 54,
    FunctionEnd = 56,
    Variable = 59,
    Load = 61,
    Store = 62,
    AccessChain = 65,
    Decorate = 71,
    MemberDecorate = 72,
    CompositeConstruct = 80,
    CompositeExtract = 81,
    Label = 248,
    Return = 253,
}

impl SpirVOpCode {
    pub fn from_u32(v: u32) -> Self {
        match v {
            3 => Self::SourceLanguage,
            4 => Self::SourceExtension,
            5 => Self::Name,
            6 => Self::MemberName,
            11 => Self::ExtInstImport,
            14 => Self::MemoryModel,
            15 => Self::EntryPoint,
            17 => Self::Capability,
            19 => Self::TypeVoid,
            21 => Self::TypeInt,
            22 => Self::TypeFloat,
            23 => Self::TypeVector,
            28 => Self::TypeArray,
            30 => Self::TypeStruct,
            32 => Self::TypePointer,
            33 => Self::TypeFunction,
            43 => Self::Constant,
            44 => Self::ConstantComposite,
            54 => Self::Function,
            56 => Self::FunctionEnd,
            59 => Self::Variable,
            61 => Self::Load,
            62 => Self::Store,
            65 => Self::AccessChain,
            71 => Self::Decorate,
            72 => Self::MemberDecorate,
            80 => Self::CompositeConstruct,
            81 => Self::CompositeExtract,
            248 => Self::Label,
            253 => Self::Return,
            _ => todo!("{:?}", v),
        }
    }
}

#[derive(Debug, Default, Clone)]
#[repr(u32)]
pub enum FunctionControl {
    #[default]
    None = 0x0,
    Inline = 0x1,
    DontInline = 0x2,
    Pure = 0x4,
    Const = 0x8,
    OptNoneEXT = 0x10000,
}

impl FunctionControl {
    pub fn from_u32(v: u32) -> Self {
        match v {
            0x0 => Self::None,
            0x1 => Self::Inline,
            0x2 => Self::DontInline,
            0x4 => Self::Pure,
            0x8 => Self::Const,
            0x10000 => Self::OptNoneEXT,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SpirVCapability {
    #[default]
    Matrix = 0,
    Shader,
    Geometry,
    Tessellation,
    Addresses,
    Linkage,
    Kernel,
    Vector16,
    Float16Buffer,
    Float16,
    Float64,
    Int64,
    Int64Atomics,
    ImageBasic,
    ImageReadWrite,
    ImageMipmap,
    Pipes = 17,
    Groups,
    DeviceEnqueue,
    LiteralSampler,
    AtomicStorage,
    Int16,
    TessellationPointSize,
    GeometryPointSize,
    ImageGatherExtended,
    StorageImageMultisample = 27,
    UniformBufferArrayDynamicIndexing,
    SampledImageArrayDynamicIndexing,
    StorageBufferArrayDynamicIndexing,
    StorageImageArrayDynamicIndexing,
    ClipDistance,
    CullDistance,
    ImageCubeArray,
    SampleRateShading,
    ImageRect,
    SampledRect,
    GenericPointer,
    Int8,
    InputAttachment,
    SparseResidency,
    MinLod,
    Sampled1D,
    Image1D,
    SampledCubeArray,
    SampledBuffer,
    ImageBuffer,
    ImageMSArray,
    StorageImageExtendedFormats,
    ImageQuery,
    DerivativeControl,
    InterpolationFunction,
    TransformFeedback,
    GeometryStreams,
    StorageImageReadWithoutFormat,
    StorageImageWriteWithoutFormat,
    MultiViewport,
    SubgroupDispatch,
    NamedBarrier,
    PipeStorage,
    GroupNonUniform,
    GroupNonUniformVote,
    GroupNonUniformArithmetic,
    GroupNonUniformBallot,
    GroupNonUniformShuffle,
    GroupNonUniformShuffleRelative,
    GroupNonUniformClustered,
    GroupNonUniformQuad,
    ShaderLayer,
    ShaderViewportIndex,
    UniformDecoration,
    CoreBuiltinsARM = 4165,
    TileImageColorReadAccessEXT,
    TileImageDepthReadAccessEXT,
    TileImageStencilReadAccessEXT,
    TensorsARM = 4174,
    StorageTensorArrayDynamicIndexingARM,
    StorageTensorArrayNonUniformIndexingARM,
    GraphARM = 4191,
    CooperativeMatrixLayoutsARM = 4201,
    Float8EXT = 4212,
    Float8CooperativeMatrixEXT,
    FragmentShadingRateKHR = 4422,
    SubgroupBallotKHR,
    DrawParameters = 4427,
    WorkgroupMemoryExplicitLayoutKHR,
    WorkgroupMemoryExplicitLayout8BitAccessKHR,
    WorkgroupMemoryExplicitLayout16BitAccessKHR,
    SubgroupVoteKHR,
    StorageBuffer16BitAccess = 4433,
    UniformAndStorageBuffer16BitAccess,
    StoragePushConstant16,
    StorageInputOutput16,
    DeviceGroup,
    MultiView = 4439,
    VariablePointersStorageBuffer = 4441,
    VariablePointers,
    AtomicStorageOps = 4445,
    SampleMaskPostDepthCoverage = 4447,
    StorageBuffer8BitAccess,
    UniformAndStorageBuffer8BitAccess,
    StoragePushConstant8,
    DenormPreserve = 4464,
    DenormFlushToZero,
    SignedZeroInfNanPreserve,
    RoundingModeRTE,
    RoundingModeRTZ,
    RayQueryProvisionalKHR = 4471,
    RayQueryKHR,
    UntypedPointersKHR,
    RayTraversalPrimitiveCullingKHR = 4478,
    RayTracingKHR,
    TextureSampleWeightedQCOM = 4484,
    TextureBoxFilterQCOM,
    TextureBlockMatchQCOM,
    TileShadingQCOM = 4495,
    CooperativeMatrixConversionQCOM,
    TextureBlockMatch2QCOM = 4498,
    Float16ImageAMD = 5008,
    ImageGatherBiasLodAMD,
    FragmentMaskAMD,
    StencilExportEXT = 5013,
    ImageReadWriteLodAMD = 5015,
    Int64ImageEXT,
    ShaderClockKHR = 5055,
    ShaderEnqueueAMDX = 5067,
    QuadControlKHR = 5087,
    Int4TypeINTEL = 5112,
    Int4CooperativeMatrixINTEL = 5114,
    BFloat16TypeKHR = 5116,
    BFloat16DotProductKHR,
    BFloat16CooperativeMatrixKHR,
    SampleMaskOverrideCoverageNV = 5249,
    GeometryShaderPassthroughNV = 5251,
    ShaderViewportIndexLayerEXT = 5254,
    ShaderViewportMaskNV = 5255,
    ShaderStereoViewNV = 5259,
    PerViewAttributesNV,
    FragmentFullyCoveredEXT = 5265,
    MeshShadingNV,
    ImageFootprintNV = 5282,
    MeshShadingEXT,
    FragmentBarycentricKHR,
    ComputeDerivativeGroupQuadsKHR = 5288,
    FragmentDensityEXT = 5291,
    GroupNonUniformPartitionedNV = 5297,
    ShaderNonUniform = 5301,
    RuntimeDescriptorArray,
    InputAttachmentArrayDynamicIndexing,
    UniformTexelBufferArrayDynamicIndexing,
    StorageTexelBufferArrayDynamicIndexing,
    UniformBufferArrayNonUniformIndexing,
    SampledImageArrayNonUniformIndexing,
    StorageBufferArrayNonUniformIndexing,
    StorageImageArrayNonUniformIndexing,
    InputAttachmentArrayNonUniformIndexing,
    UniformTexelBufferArrayNonUniformIndexing,
    StorageTexelBufferArrayNonUniformIndexing,
    RayTracingPositionFetchKHR = 5336,
    RayTracingNV = 5340,
    RayTracingMotionBlurNV,
    VulkanMemoryModel = 5345,
    VulkanMemoryModelDeviceScope,
    PhysicalStorageBufferAddresses,
    ComputeDerivativeGroupLinearKHR = 5350,
    RayTracingProvisionalKHR = 5353,
    CooperativeMatrixNV = 5357,
    FragmentShaderSampleInterlockEXT = 5363,
    FragmentShaderShadingRateInterlockEXT = 5372,
    ShaderSMBuiltinsNV,
    FragmentShaderPixelInterlockEXT = 5378,
    DemoteToHelperInvocation,
    DisplacementMicromapNV,
    RayTracingOpacityMicromapEXT,
    ShaderInvocationReorderNV = 5383,
    BindlessTextureNV = 5390,
    RayQueryPositionFetchKHR,
    CooperativeVectorNV = 5394,
    AtomicFloat16VectorNV = 5404,
    RayTracingDisplacementMicromapNV = 5409,
    RawAccessChainsNV = 5414,
    RayTracingSpheresGeometryNV = 5418,
    RayTracingLinearSweptSpheresGeometryNV,
    CooperativeMatrixReductionsNV = 5430,
    CooperativeMatrixConversionsNV,
    CooperativeMatrixPerElementOperationsNV,
    CooperativeMatrixTensorAddressingNV,
    CooperativeMatrixBlockLoadsNV,
    CooperativeVectorTrainingNV,
    RayTracingClusterAccelerationStructureNV = 5437,
    TensorAddressingNV = 5439,
    SubgroupShuffleINTEL = 5568,
    SubgroupBufferBlockIOINTEL,
    SubgroupImageBlockIOINTEL = 5570,
    SubgroupImageMediaBlockIOINTEL = 5579,
    RoundToInfinityINTEL = 5582,
    FloatingPointModeINTEL,
    IntegerFunctions2INTEL,
    FunctionPointersINTEL = 5603,
    IndirectReferencesINTEL,
    AsmINTEL = 5606,
    AtomicFloat32MinMaxEXT = 5612,
    AtomicFloat64MinMaxEXT,
    AtomicFloat16MinMaxEXT = 5616,
    VectorComputeINTEL,
    VectorAnyINTEL = 5619,
    ExpectAssumeKHR = 5629,
    SubgroupAvcMotionEstimationINTEL = 5696,
    SubgroupAvcMotionEstimationIntraINTEL,
    SubgroupAvcMotionEstimationChromalINTEL,
    VariableLengthArrayINTEL = 5817,
    FunctionFloatControlINTEL = 5821,
    FPGAMemoryAttributesINTEL = 5824,
    FPFastMathModelINTEL = 5837,
    ArbitraryPrecisionIntegersINTEL = 5844,
    ArbitraryPrecisionFloatingPointINTEL,
    UnstructuredLoopControlsINTEL = 5886,
    FPGALoopControlsINTEL = 5888,
    KernelAttributesINTEL = 5892,
    FPGAKernelAttributesINTEL = 5897,
    FPGAMemoryAccessesINTEL,
    FPGAClusterAttributesINTEL = 5904,
    LoopFuseINTEL = 5906,
    FPGADSPControlINTEL = 5908,
    MemoryAccessAliasingINTEL = 5910,
    FPGAInvocationPipeliningAttributesINTEL = 5916,
    FPGABufferLocationINTEL = 5920,
    ArbitraryPrecisionFixedPointINTEL = 5922,
    USMStorageClassesINTEL = 5935,
    RuntimeAlignedAttributeINTEL = 5939,
    IOPipesINTEL = 5943,
    BlockingPipesINTEL = 5945,
    FPGARegINTEL = 5948,
    DotProductInputAll = 6016,
    DotProductInput4x8Bit,
    DotProductInput4x8BitPacked,
    DotProduct,
    RayCullMaskKHR = 6020,
    CooperativeMatrixKHR = 6022,
    ReplicatedCompositesEXT = 6024,
    BitInstructions,
    GroupNonUniformRotateKHR,
    FloatControls2 = 6029,
    AtomicFloat32AddEXT = 6033,
    AtomicFloat64AddEXT,
    LongCompositesINTEL = 6089,
    OptNoneEXT = 6094,
    AtomicFloat16AddEXT,
    DebugInfoModuleINTEL = 6114,
    SplitBarrierINTEL,
    ArithmeticFenceEXT = 6141,
    FPGAClusterAttributesV2INTEL = 6144,
    FPGAKernelAttributesV2INTEL = 6150,
    TaskSequenceINTEL = 6161,
    FPMaxErrorINTEL,
    FPGALatencyControlINTEL = 6169,
    FPGAArgumentInterfacesINTEL = 6171,
    GlobalVariableHostAccessINTEL = 6174,
    GlobalVariableFPGADecorationsINTEL = 6189,
    SubgroupBufferPrefetchINTEL = 6220,
    Subgroup2DBlockIOINTEL = 6228,
    Subgroup2DBlockTransformINTEL,
    Subgroup2DBlockTransposeINTEL,
    SubgroupMatrixMultiplyAccumulateINTEL = 6236,
    TernaryBitwiseFunctionINTEL = 6241,
    SpecConditionalINTEL = 6245,
    FunctionVariantsINTEL = 6246,
    GroupUniformArithmeticKHR = 6400,
    TensorFloat32RoundingINTEL = 6425,
    MaskedGatherScatterINTEL = 6427,
    CacheControlsINTEL = 6441,
    RegisterLimitsINTEL = 6460,
    BindlessImagesINTEL = 6528,
}
