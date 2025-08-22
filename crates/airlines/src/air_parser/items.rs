use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;

use crate::llvm_bitcode::{AttributeKindCode, Block, Fields};

#[derive(Debug, Default)]
pub struct AIRFile {
    pub items: Vec<AIRItem>,
}

#[derive(Debug, Clone)]
pub enum AIRItem {
    IdentificationBlock(AIRIdentificationBlock),
    Module(AIRModule),
}

#[derive(Debug, Default, Clone)]
pub struct TableString {
    pub offset: u64,
    pub size: u64,
    pub content: String,
}

#[derive(Debug, Default, Clone)]
pub struct AIRGlobalVariable {
    pub name: Rc<RefCell<TableString>>,
    pub ty: AIRType,
    pub is_const: bool,
    pub initializer_id: u64,
    pub linkage: LinkageCode,
    pub alignment: u64,
    pub section_index: u64,
    pub visibility: VisibilityCode,
    pub thread_local: ThreadLocalCode,
    pub unnamed_addr: UnnamedAddrCode,
    pub dll_storage_class: DllStorageClassCode,
    pub comdat: u64,
    pub attribute_index: u64,
    pub preemption_specifier: PreemptionSpecifierCode,
}

#[derive(Debug, Default, Clone)]
pub struct AIRConstant {
    pub ty: AIRType,
    pub value: AIRConstantValue,
}

#[derive(Debug, Default, Clone)]
pub enum AIRConstantValue {
    #[default]
    Null,
    Integer(u64),
    Float32(f32),
    Aggregate(Vec<AIRConstant>),
    Array(Vec<AIRConstantValue>),
    Pointer(u64),
}

#[derive(Debug, Default, Clone)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum LinkageCode {
    #[default]
    EXTERNAL = 0,
    WEAK,
    APPENDING,
    INTERNAL,
    LINK_ONCE,
    DLL_IMPORT,
    DLL_EXPORT,
    EXTERN_WEAK,
    COMMON,
    PRIVATE,
    WEAK_ODR,
    LINK_ONCE_ODR,
    AVAILABLE_EXTERNALLY,
    DEPRECATED1,
    DEPRECATED2,
}

impl LinkageCode {
    pub fn from_u64(v: u64) -> Self {
        match v {
            0 => Self::EXTERNAL,
            1 => Self::WEAK,
            2 => Self::APPENDING,
            3 => Self::INTERNAL,
            4 => Self::LINK_ONCE,
            5 => Self::DLL_IMPORT,
            6 => Self::DLL_EXPORT,
            7 => Self::EXTERN_WEAK,
            8 => Self::COMMON,
            9 => Self::PRIVATE,
            10 => Self::WEAK_ODR,
            11 => Self::LINK_ONCE_ODR,
            12 => Self::AVAILABLE_EXTERNALLY,
            13 => Self::DEPRECATED1,
            14 => Self::DEPRECATED2,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Default, Clone)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum VisibilityCode {
    #[default]
    DEFAULT = 0,
    HIDDEN,
    PROTECTED,
}

impl VisibilityCode {
    pub fn from_u64(v: u64) -> Self {
        match v {
            0 => Self::DEFAULT,
            1 => Self::HIDDEN,
            2 => Self::PROTECTED,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Default, Clone)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum ThreadLocalCode {
    #[default]
    NOT_THREAD_LOCAL = 0,
    THREAD_LOCAL,
    LOCAL_DYNAMIC,
    INITIAL_EXEC,
    LOCAL_EXEC,
}

impl ThreadLocalCode {
    pub fn from_u64(v: u64) -> Self {
        match v {
            0 => Self::NOT_THREAD_LOCAL,
            1 => Self::THREAD_LOCAL,
            2 => Self::LOCAL_DYNAMIC,
            3 => Self::INITIAL_EXEC,
            4 => Self::LOCAL_EXEC,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Default, Clone)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum UnnamedAddrCode {
    #[default]
    NOT_UNNAMED_ADDR = 0,
    UNNAMED_ADDR,
    LOCAL_UNNAMED_ADDR,
}

impl UnnamedAddrCode {
    pub fn from_u64(v: u64) -> Self {
        match v {
            0 => Self::NOT_UNNAMED_ADDR,
            1 => Self::UNNAMED_ADDR,
            2 => Self::LOCAL_UNNAMED_ADDR,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Default, Clone)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum DllStorageClassCode {
    #[default]
    DEFAULT = 0,
    DLL_IMPORT,
    DLL_EXPORT,
}

impl DllStorageClassCode {
    pub fn from_u64(v: u64) -> Self {
        match v {
            0 => Self::DEFAULT,
            1 => Self::DLL_IMPORT,
            2 => Self::DLL_EXPORT,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Default, Clone)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum PreemptionSpecifierCode {
    #[default]
    DSO_PREEMPTABLE = 0,
    DSO_LOCAL,
}

impl PreemptionSpecifierCode {
    pub fn from_u64(v: u64) -> Self {
        match v {
            0 => Self::DSO_PREEMPTABLE,
            1 => Self::DSO_LOCAL,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Default, Clone)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum CallingConventionCode {
    #[default]
    C = 0,
    FAST = 8,
    COLD = 9,
    ANY_REG = 13,
    PRESERVE_MOST = 14,
    PRESERVE_ALL = 15,
    SWIFT = 16,
    CXX_FAST_TLS = 17,
    TAIL = 18,
    CFGUARD_CHECK = 19,
    SWIFT_TAIL = 20,
    X86_STDCALL = 64,
    X86_FASTCALL = 65,
    ARM_APCS = 66,
    ARM_AAPCS = 67,
    ARM_AAPCS_VFP = 68,
}

impl CallingConventionCode {
    pub fn from_u64(v: u64) -> Self {
        match v {
            0 => Self::C,
            8 => Self::FAST,
            9 => Self::COLD,
            13 => Self::ANY_REG,
            14 => Self::PRESERVE_MOST,
            15 => Self::PRESERVE_ALL,
            16 => Self::SWIFT,
            17 => Self::CXX_FAST_TLS,
            18 => Self::TAIL,
            19 => Self::CFGUARD_CHECK,
            20 => Self::SWIFT_TAIL,
            64 => Self::X86_STDCALL,
            65 => Self::X86_FASTCALL,
            66 => Self::ARM_APCS,
            67 => Self::ARM_AAPCS,
            68 => Self::ARM_AAPCS_VFP,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AIRFunctionSignature {
    pub name: Rc<RefCell<TableString>>,
    pub ty: AIRType,
    pub calling_convention: CallingConventionCode,
    pub is_proto: bool,
    pub linkage: LinkageCode,
    pub attr_entry: Option<AIRAttrEntry>,
    pub alignment: u64,
    pub section_index: u64,
    pub visibility: VisibilityCode,
    pub gc_index: u64,
    pub unnamed_addr: UnnamedAddrCode,
    pub prologue_data_index: u64,
    pub comdat: u64,
    pub prefix_data_index: u64,
    pub personality_fn_index: u64,
    pub preemption_specifier: PreemptionSpecifierCode,
}

#[derive(Debug, Default, Clone)]
#[allow(non_camel_case_types)]
pub enum UndiscoveredData {
    #[default]
    NONE,
    VSTOFFSET(u64),
    INDEX_OFFSET(u64),
}

#[derive(Debug, Default, Clone)]
pub struct AIRMetadataKind {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Default, Clone)]
pub enum AIRMetadataConstant {
    #[default]
    None,
    Value(AIRConstantValue),
    Pointer(u64),
    Node(String, Vec<u64>),
}

#[derive(Debug, Default, Clone)]
pub struct AIRModule {
    pub version: u64,
    pub triple: String,
    pub data_layout: String,
    pub source_filename: String,
    pub types: Vec<AIRType>,
    pub attributes: HashMap<u64, AIRAttribute>,
    pub entry_table: HashMap<u64, AIRAttrEntry>,
    pub string_table: Vec<Rc<RefCell<TableString>>>,
    pub global_variables: HashMap<u64, AIRGlobalVariable>,
    pub function_signatures: HashMap<u64, AIRFunctionSignature>,
    pub constants: HashMap<u64, AIRConstant>,
    pub undiscovered_data: Vec<UndiscoveredData>,
    pub metadata_kind_table: HashMap<u64, AIRMetadataKind>,
    pub metadata_strings: Vec<String>,
    pub metadata_constants: HashMap<u64, AIRMetadataConstant>,
    pub operand_bundle_tags: Vec<String>,
    pub sync_scope_names: Vec<String>,
    pub max_global_id: u64,
}

impl AIRModule {
    pub fn parse_global_variable(&mut self, fields: Fields) {
        let string_offset = fields[0];
        let string_size = fields[1];

        self.string_table.push(Rc::new(RefCell::new(TableString {
            offset: string_offset,
            size: string_size,
            content: String::new(),
        })));

        let name = self.string_table.last().unwrap().clone();
        let ty = self.types[fields[2] as usize].clone();
        let is_const = fields[3] != 0;

        let initializer_id = fields[4];

        let linkage = LinkageCode::from_u64(fields[5]);
        let alignment = match fields[7].checked_sub(1) {
            Some(result) => 2_u64.pow(result as u32),
            None => 0,
        };
        // TODO: Parse section (fields[7]) correctly.
        let section_index = fields[7];

        let visibility = VisibilityCode::from_u64(fields[8]);
        let thread_local = ThreadLocalCode::from_u64(fields[9]);
        let unnamed_addr = UnnamedAddrCode::from_u64(fields[10]);
        let dll_storage_class = DllStorageClassCode::from_u64(fields[11]);

        // TODO: Parse comdat (fields[12]) correctly.
        let comdat = fields[12];

        let attribute_index = fields[13];
        let preemption_specifier = PreemptionSpecifierCode::from_u64(fields[14]);

        self.global_variables.insert(
            self.max_global_id,
            AIRGlobalVariable {
                name,
                ty,
                is_const,
                initializer_id,
                linkage,
                alignment,
                section_index,
                visibility,
                thread_local,
                unnamed_addr,
                dll_storage_class,
                comdat,
                attribute_index,
                preemption_specifier,
            },
        );

        self.max_global_id += 1;
    }

    pub fn parse_function_signature(&mut self, fields: Fields) {
        let string_offset = fields[0];
        let string_size = fields[1];

        self.string_table.push(Rc::new(RefCell::new(TableString {
            offset: string_offset,
            size: string_size,
            content: String::new(),
        })));

        let name = self.string_table.last().unwrap().clone();
        let ty = self.types[fields[2] as usize].clone();
        let calling_convention = CallingConventionCode::from_u64(fields[3]);
        let is_proto = fields[4] != 0;
        let linkage = LinkageCode::from_u64(fields[5]);

        let attr_entry = match self.entry_table.get(&fields[6]) {
            Some(entry) => Some(entry.clone()),
            None => None,
        };

        let alignment = match fields[7].checked_sub(1) {
            Some(result) => 2_u64.pow(result as u32),
            None => 0,
        };

        // TODO: Parse section (fields[8]) correctly.
        let section_index = fields[8];
        let visibility = VisibilityCode::from_u64(fields[9]);

        // TODO: Parse gc (fields[10]) correctly.
        let gc_index = fields[10];
        let unnamed_addr = UnnamedAddrCode::from_u64(fields[11]);

        // TODO: Parse prologue_data (fields[12]) correctly.
        let prologue_data_index = fields[12];

        // TODO: Parse comdat (fields[13]) correctly.
        let comdat = fields[13];

        // TODO: Parse prefix_data (fields[14]) correctly.
        let prefix_data_index = fields[14];

        // TODO: Parse personality_fn (fields[15]) correctly.
        let personality_fn_index = fields[15];

        let preemption_specifier = PreemptionSpecifierCode::from_u64(fields[16]);

        self.function_signatures.insert(
            self.max_global_id,
            AIRFunctionSignature {
                name,
                ty,
                calling_convention,
                is_proto,
                linkage,
                attr_entry,
                alignment,
                section_index,
                visibility,
                gc_index,
                unnamed_addr,
                prologue_data_index,
                comdat,
                prefix_data_index,
                personality_fn_index,
                preemption_specifier,
            },
        );
    }
}

#[derive(Debug, Default, Clone)]
pub struct AIRAttrEntry {
    pub groups: Vec<u64>,
}

#[derive(Debug, Default, Clone)]
pub struct AIRAttribute {
    pub id: u64,
    pub paramidx: u64,
    pub properties: Vec<AIRAttrProperties>,
}

#[derive(Debug, Default, Clone)]
pub struct AIRArrayType {
    pub size: u64,
    pub element_type: Box<AIRType>,
}

#[derive(Debug, Default, Clone)]
pub struct AIRVectorType {
    pub size: u64,
    pub element_type: Box<AIRType>,
}

#[derive(Debug, Default, Clone)]
pub struct AIRStructType {
    pub name: String,
    pub is_packed: bool,
    pub elements: Vec<AIRType>,
}

#[derive(Debug, Default, Clone)]
pub struct AIRFunctionType {
    pub vararg: u64,
    pub return_type: Box<AIRType>,
    pub params: Vec<AIRType>,
}

#[derive(Debug, Default, Clone)]
pub enum AIRType {
    #[default]
    Void,
    Float,
    Integer(u64),
    Pointer(u64, Box<AIRType>),
    Array(AIRArrayType),
    Vector(AIRVectorType),
    Struct(AIRStructType),
    Function(AIRFunctionType),
    Metadata,
}

#[derive(Debug, Default, Clone)]
pub struct AIRIdentificationBlock {
    pub string: String,
    pub epoch: Vec<u64>,
}

#[derive(Debug, Clone)]
pub enum AIRAttrProperties {
    WellKnown(AttributeKindCode),
    WithIntValue(AttributeKindCode, u64),
    StringAttribute(String),
    WithStringValue(String, String),
}
