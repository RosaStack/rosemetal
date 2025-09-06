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
    CompositeConstruct(SpirVVariableId, SpirVCompositeConstruct),
    Return,
    Function(SpirVVariableId, SpirVFunction),
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

#[derive(Debug, Default, Clone)]
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

#[derive(Debug, Default, Clone, Copy)]
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

#[derive(Debug, Default, Clone)]
pub struct SpirVConstant {
    pub type_id: SpirVVariableId,
    pub value: SpirVConstantValue,
}

#[derive(Debug, Default, Clone)]
pub struct SpirVConstantComposite {
    pub type_id: SpirVVariableId,
    pub values: Vec<SpirVVariableId>,
}

#[derive(Debug, Default, Clone)]
pub enum SpirVConstantValue {
    #[default]
    Undefined,
    SignedInteger(i64),
    UnsignedInteger(u64),
    Float32(f32),
    Float64(f64),
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

#[derive(Debug, Default, Clone)]
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum SpirVCapability {
    #[default]
    Matrix,
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
    Pipes,
    Groups,
    DeviceEnqueue,
    LiteralSampler,
    AtomicStorage,
    Int16,
    TessellationPointSize,
    GeometryPointSize,
    ImageGatherExtended,
    StorageImageMultisample,
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
    CoreBuiltinsARM,
    TileImageColorReadAccessEXT,
    TileImageDepthReadAccessEXT,
    TileImageStencilReadAccessEXT,
    TensorsARM,
    StorageTensorArrayDynamicIndexingARM,
    StorageTensorArrayNonUniformIndexingARM,
    GraphARM,
    CooperativeMatrixLayoutsARM,
    Float8EXT,
    Float8CooperativeMatrixEXT,
    FragmentShadingRateKHR,
    SubgroupBallotKHR,
    DrawParameters,
    WorkgroupMemoryExplicitLayoutKHR,
    WorkgroupMemoryExplicitLayout8BitAccessKHR,
    WorkgroupMemoryExplicitLayout16BitAccessKHR,
    SubgroupVoteKHR,
    StorageBuffer16BitAccess,
    UniformAndStorageBuffer16BitAccess,
    StoragePushConstant16,
    StorageInputOutput16,
    DeviceGroup,
    MultiView,
    VariablePointersStorageBuffer,
    VariablePointers,
    AtomicStorageOps,
    SampleMaskPostDepthCoverage,
    StorageBuffer8BitAccess,
    UniformAndStorageBuffer8BitAccess,
    StoragePushConstant8,
    DenormPreserve,
    DenormFlushToZero,
    SignedZeroInfNanPreserve,
    RoundingModeRTE,
    RoundingModeRTZ,
    RayQueryProvisionalKHR,
    RayQueryKHR,
    UntypedPointersKHR,
    RayTraversalPrimitiveCullingKHR,
    RayTracingKHR,
    TextureSampleWeightedQCOM,
    TextureBoxFilterQCOM,
    TextureBlockMatchQCOM,
    TileShadingQCOM,
    CooperativeMatrixConversionQCOM,
    TextureBlockMatch2QCOM,
    Float16ImageAMD,
    ImageGatherBiasLodAMD,
    FragmentMaskAMD,
    StencilExportEXT,
    ImageReadWriteLodAMD,
    Int64ImageEXT,
    ShaderClockKHR,
    ShaderEnqueueAMDX,
    QuadControlKHR,
    Int4TypeINTEL,
    Int4CooperativeMatrixINTEL,
    BFloat16TypeKHR,
    BFloat16DotProductKHR,
    BFloat16CooperativeMatrixKHR,
    SampleMaskOverrideCoverageNV,
    GeometryShaderPassthroughNV,
    ShaderViewportIndexLayerEXT,
    ShaderViewportMaskNV,
    ShaderStereoViewNV,
    PerViewAttributesNV,
    FragmentFullyCoveredEXT,
    MeshShadingNV,
    ImageFootprintNV,
    MeshShadingEXT,
    FragmentBarycentricKHR,
    ComputeDerivativeGroupQuadsKHR,
    FragmentDensityEXT,
    GroupNonUniformPartitionedNV,
    ShaderNonUniform,
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
    RayTracingPositionFetchKHR,
    RayTracingNV,
    RayTracingMotionBlurNV,
    VulkanMemoryModel,
    VulkanMemoryModelDeviceScope,
    PhysicalStorageBufferAddresses,
    ComputeDerivativeGroupLinearKHR,
    RayTracingProvisionalKHR,
    CooperativeMatrixNV,
    FragmentShaderSampleInterlockEXT,
    FragmentShaderShadingRateInterlockEXT,
    ShaderSMBuiltinsNV,
    FragmentShaderPixelInterlockEXT,
    DemoteToHelperInvocation,
    DisplacementMicromapNV,
    RayTracingOpacityMicromapEXT,
    ShaderInvocationReorderNV,
    BindlessTextureNV,
    RayQueryPositionFetchKHR,
    CooperativeVectorNV,
    AtomicFloat16VectorNV,
    RayTracingDisplacementMicromapNV,
    RawAccessChainsNV,
    RayTracingSpheresGeometryNV,
    RayTracingLinearSweptSpheresGeometryNV,
    CooperativeMatrixReductionsNV,
    CooperativeMatrixConversionsNV,
    CooperativeMatrixPerElementOperationsNV,
    CooperativeMatrixTensorAddressingNV,
    CooperativeMatrixBlockLoadsNV,
    CooperativeVectorTrainingNV,
    RayTracingClusterAccelerationStructureNV,
    TensorAddressingNV,
    SubgroupShuffleINTEL,
    SubgroupBufferBlockIOINTEL,
    SubgroupImageBlockIOINTEL,
    SubgroupImageMediaBlockIOINTEL,
    RoundToInfinityINTEL,
    FloatingPointModeINTEL,
    IntegerFunctions2INTEL,
    FunctionPointersINTEL,
    IndirectReferencesINTEL,
    AsmINTEL,
    AtomicFloat32MinMaxEXT,
    AtomicFloat64MinMaxEXT,
    AtomicFloat16MinMaxEXT,
    VectorComputeINTEL,
    VectorAnyINTEL,
    ExpectAssumeKHR,
    SubgroupAvcMotionEstimationINTEL,
    SubgroupAvcMotionEstimationIntraINTEL,
    SubgroupAvcMotionEstimationChromalINTEL,
    VariableLengthArrayINTEL,
    FunctionFloatControlINTEL,
    FPGAMemoryAttributesINTEL,
    FPFastMathModelINTEL,
    ArbitraryPrecisionIntegersINTEL,
    ArbitraryPrecisionFloatingPointINTEL,
    UnstructuredLoopControlsINTEL,
    FPGALoopControlsINTEL,
    KernelAttributesINTEL,
    FPGAKernelAttributesINTEL,
    FPGAMemoryAccessesINTEL,
    FPGAClusterAttributesINTEL,
    LoopFuseINTEL,
    FPGADSPControlINTEL,
    MemoryAccessAliasingINTEL,
    FPGABufferLocationINTEL,
    ArbitraryPrecisionFixedPointINTEL,
    USMStorageClassesINTEL,
    RuntimeAlignedAttributeINTEL,
    IOPipesINTEL,
    BlockingPipesINTEL,
    FPGARegINTEL,
    DotProductInputAll,
    DotProductInput4x8Bit,
    DotProductInput4x8BitPacked,
    DotProduct,
    RayCullMaskKHR,
    CooperativeMatrixKHR,
    ReplicatedCompositesEXT,
    BitInstructions,
    GroupNonUniformRotateKHR,
    FloatControls2,
    AtomicFloat32AddEXT,
    AtomicFloat64AddEXT,
    LongCompositesINTEL,
    OptNoneEXT,
    AtomicFloat16AddEXT,
    DebugInfoModuleINTEL,
    SplitBarrierINTEL,
    ArithmeticFenceEXT,
    FPGAClusterAttributesV2INTEL,
    FPGAKernelAttributesV2INTEL,
    TaskSequenceINTEL,
    FPMaxErrorINTEL,
    FPGALatencyControlINTEL,
    FPGAArgumentInterfacesINTEL,
    GlobalVariableHostAccessINTEL,
    GlobalVariableFPGADecorationsINTEL,
    SubgroupBufferPrefetchINTEL,
    Subgroup2DBlockIOINTEL,
    Subgroup2DBlockTransformINTEL,
    Subgroup2DBlockTransposeINTEL,
    SubgroupMatrixMultiplyAccumulateINTEL,
    TernaryBitwiseFunctionINTEL,
    SpecConditionalINTEL,
    FunctionVariantsINTEL,
    GroupUniformArithmeticKHR,
    TensorFloat32RoundingINTEL,
    MaskedGatherScatterINTEL,
    CacheControlsINTEL,
    RegisterLimitsINTEL,
    BindlessImagesINTEL,
}
