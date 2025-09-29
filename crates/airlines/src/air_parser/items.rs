use std::collections::HashMap;

use anyhow::{Result, anyhow};

use crate::llvm_bitcode::{AttributeKindCode, CastOpCode, Fields, GEPNoWrapFlags};

#[derive(Debug, Default)]
pub struct AirFile {
    pub items: Vec<AirItem>,
}

#[derive(Debug, Clone)]
pub enum AirItem {
    IdentificationBlock(AirIdentificationBlock),
    Module(AirModule),
    SymTabBlock(AirSymTabBlock),
    StringTable(AirStringTable),
}

#[derive(Debug, Default, Clone)]
pub struct AirStringTable {
    pub strings: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct AirSymTabBlock {
    pub blobs: Vec<AirBlob>,
}

#[derive(Debug, Default, Clone)]
pub struct AirBlob {
    pub content: Vec<u8>,
}

#[derive(Debug, Default, Clone)]
pub struct TableString {
    pub offset: u64,
    pub size: u64,
    pub content: String,
}

#[derive(Debug, Default, Clone)]
pub struct AirGlobalVariable {
    pub name: TableStringId,
    pub type_id: AirTypeId,
    pub is_const: bool,
    pub initializer: AirConstantId,
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
pub struct AirConstant {
    pub ty: AirTypeId,
    pub value: AirConstantValue,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum AirConstantValue {
    #[default]
    Null,
    Undefined,
    Poison,
    Unresolved(u64),
    Integer(u64),
    Float32(f32),
    Aggregate(Vec<AirValueId>),
    Array(Vec<AirConstantValue>),
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
pub struct AirFunctionSignature {
    pub global_id: AirFunctionSignatureId,
    pub name: TableStringId,
    pub ty: AirFunctionType,
    pub calling_convention: CallingConventionCode,
    pub is_proto: bool,
    pub linkage: LinkageCode,
    pub attr_entry: Option<AirAttrEntry>,
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
pub struct AirMetadataKind {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Default, Clone)]
pub enum AirMetadataConstant {
    #[default]
    None,
    Value(AirValue),
    Pointer(u64),
    Node(Vec<u64>),
    String(String),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum AirValue {
    #[default]
    Empty,
    GlobalVariable(AirGlobalVariableId),
    Constant(AirConstantId),
    Function(AirFunctionSignatureId),
    Argument(AirLocal),
    Cast(AirCast),
    GetElementPtr(AirGetElementPtr),
    Load(AirLoad),
    ShuffleVec(AirShuffleVec),
    InsertVal(AirInsertVal),
    Return(AirReturn),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct AirReturn {
    pub value: AirValueId,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct AirInsertVal {
    pub value1: AirValueId,
    pub value2: AirValueId,
    pub insert_value_idx: u64,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct AirShuffleVec {
    pub vec1: AirValueId,
    pub vec2: AirValueId,
    pub mask: AirValueId,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct AirLoad {
    pub op: AirValueId,
    pub ty: AirType,
    pub alignment: u64,
    pub vol: u64,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct AirGetElementPtr {
    pub no_wrap_flags: GEPNoWrapFlags,
    pub ty: AirType,
    pub base_ptr_value: AirValueId,
    pub indices: Vec<AirValueId>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct AirCast {
    pub value: AirValueId,
    pub cast_to_type: AirType,
    pub cast_code: CastOpCode,
}

#[derive(Debug, Default, Clone)]
pub struct AirFunctionBody {
    pub signature: AirFunctionSignatureId,
    pub contents: Vec<AirValueId>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct AirLocal {
    pub id: u64,
    pub type_id: AirTypeId,
    pub value: Option<AirConstantValue>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TableStringId(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AirGlobalVariableId(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AirConstantId(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AirFunctionSignatureId(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AirValueId(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AirTypeId(pub u64);

#[derive(Debug, Default, Clone)]
pub struct AirMetadataNamedNode {
    pub name: String,
    pub operands: Vec<u64>,
}

#[derive(Debug, Default, Clone)]
pub struct AirModule {
    pub version: u64,
    pub use_relative_ids: bool,
    pub triple: String,
    pub data_layout: String,
    pub source_filename: String,
    pub types: Vec<AirType>,
    pub attributes: HashMap<u64, AirAttribute>,
    pub entry_table: HashMap<u64, AirAttrEntry>,
    pub string_table: Vec<TableString>,
    pub global_variables: HashMap<AirGlobalVariableId, AirGlobalVariable>,
    pub function_signatures: Vec<AirFunctionSignature>,
    pub function_bodies: Vec<AirFunctionBody>,
    pub current_function_local_id: u64,
    pub constants: HashMap<AirConstantId, AirConstant>,
    pub max_constants_id: u64,
    pub value_list: Vec<AirValue>,
    pub undiscovered_data: Vec<UndiscoveredData>,
    pub metadata_kind_table: HashMap<u64, AirMetadataKind>,
    pub metadata_constants: HashMap<u64, AirMetadataConstant>,
    pub metadata_named_nodes: Vec<AirMetadataNamedNode>,
    pub operand_bundle_tags: Vec<String>,
    pub sync_scope_names: Vec<String>,
    pub max_global_id: u64,
}

impl AirModule {
    pub fn get_metadata_string(&self, id: u64) -> Option<String> {
        match self.metadata_constants.get(&id).unwrap() {
            AirMetadataConstant::String(string) => Some(string.clone()),
            _ => None,
        }
    }

    pub fn get_function_signature(
        &self,
        id: AirFunctionSignatureId,
    ) -> Option<&AirFunctionSignature> {
        for i in &self.function_signatures {
            if i.global_id == id {
                return Some(i);
            }
        }

        None
    }

    pub fn assign_value_to_value_list(&mut self, id: usize, value: AirValue) {
        match self.value_list.get_mut(id) {
            Some(s) => *s = value,
            None => {
                let difference = id - self.value_list.len();
                self.value_list
                    .resize(self.value_list.len() + difference + 1, AirValue::Empty);
                self.value_list[id] = value;
            }
        }
    }

    pub fn parse_global_variable(&mut self, fields: Fields) {
        let string_offset = fields[0];
        let string_size = fields[1];

        self.string_table.push(TableString {
            offset: string_offset,
            size: string_size,
            content: String::new(),
        });

        let name = TableStringId(self.string_table.len() as u64 - 1);
        let ty = AirTypeId(fields[2]);
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

        let result = AirGlobalVariable {
            name,
            type_id: ty.clone(),
            is_const,
            initializer: AirConstantId(initializer_id),
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
        };

        self.value_list
            .push(AirValue::GlobalVariable(AirGlobalVariableId(
                self.max_global_id,
            )));

        self.global_variables
            .insert(AirGlobalVariableId(self.max_global_id), result);

        self.max_global_id += 1;
    }

    pub fn parse_function_signature(&mut self, fields: Fields) -> Result<()> {
        let string_offset = fields[0];
        let string_size = fields[1];

        self.string_table.push(TableString {
            offset: string_offset,
            size: string_size,
            content: String::new(),
        });

        let name = TableStringId(self.string_table.len() as u64 - 1);
        let ty = match self.types[fields[2] as usize].clone() {
            AirType::Function(f) => f,
            _ => return Err(anyhow!("Function type not found.")),
        };
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

        self.value_list
            .push(AirValue::Function(AirFunctionSignatureId(
                self.max_global_id,
            )));

        self.function_signatures.push(AirFunctionSignature {
            global_id: AirFunctionSignatureId(self.max_global_id),
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
        });

        self.max_global_id += 1;

        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct AirAttrEntry {
    pub groups: Vec<AirAttribute>,
}

#[derive(Debug, Default, Clone)]
pub struct AirAttribute {
    pub id: u64,
    pub paramidx: u64,
    pub properties: Vec<AirAttrProperties>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AirArrayType {
    pub size: u64,
    pub element_type: AirTypeId,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AirVectorType {
    pub size: u64,
    pub element_type: AirTypeId,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AirStructType {
    pub name: String,
    pub is_packed: bool,
    pub elements: Vec<AirTypeId>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AirFunctionType {
    pub vararg: u64,
    pub return_type: AirTypeId,
    pub param_types: Vec<AirTypeId>,
    pub param_values: Vec<AirValueId>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum AirType {
    #[default]
    Void,
    Float,
    Integer(u64),
    Pointer(u64, AirTypeId),
    Array(AirArrayType),
    Vector(AirVectorType),
    Struct(AirStructType),
    Function(AirFunctionType),
    Metadata,
}

#[derive(Debug, Default, Clone)]
pub struct AirIdentificationBlock {
    pub string: String,
    pub epoch: Vec<u64>,
}

#[derive(Debug, Clone)]
pub enum AirAttrProperties {
    WellKnown(AttributeKindCode),
    WithIntValue(AttributeKindCode, u64),
    StringAttribute(String),
    WithStringValue(String, String),
}
