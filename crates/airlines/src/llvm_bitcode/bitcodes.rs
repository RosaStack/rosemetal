use anyhow::{Result, anyhow};

pub const FIRST_APPLICATION_ABBREV_ID: u64 = 4;
pub const FIRST_APPLICATION_BLOCKID: u64 = 8;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum ReservedAbbrevId {
    END_BLOCK = 0,
    ENTER_SUBBLOCK,
    DEFINE_ABBREV,
    UNABBREV_RECORD,
}

impl ReservedAbbrevId {
    pub fn from_u64(v: u64) -> Result<Self> {
        match v {
            0 => Ok(Self::END_BLOCK),
            1 => Ok(Self::ENTER_SUBBLOCK),
            2 => Ok(Self::DEFINE_ABBREV),
            3 => Ok(Self::UNABBREV_RECORD),
            _ => Err(anyhow!("'{:?}' not available.", v)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum ReservedBlockId {
    BLOCKINFO = 0,
    RESERVED1,
    RESERVED2,
    RESERVED3,
    RESERVED4,
    RESERVED5,
    RESERVED6,
    RESERVED7,
}

impl ReservedBlockId {
    pub fn from_u64(v: u64) -> Result<Self> {
        match v {
            0 => Ok(Self::BLOCKINFO),
            1 => Ok(Self::RESERVED1),
            2 => Ok(Self::RESERVED2),
            3 => Ok(Self::RESERVED3),
            4 => Ok(Self::RESERVED4),
            5 => Ok(Self::RESERVED5),
            6 => Ok(Self::RESERVED6),
            7 => Ok(Self::RESERVED7),
            _ => Err(anyhow!("'{:?}' is probably a different Block Id type.", v)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum AbbrevOpEncoding {
    Fixed = 1,
    Vbr,
    Array,
    Char6,
    Blob,
}

impl AbbrevOpEncoding {
    pub fn from_u64(v: u64) -> Result<Self> {
        match v {
            1 => Ok(Self::Fixed),
            2 => Ok(Self::Vbr),
            3 => Ok(Self::Array),
            4 => Ok(Self::Char6),
            5 => Ok(Self::Blob),
            _ => Err(anyhow!("'{:?}' is not a valid Abbrev Operand Encoding.", v)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum BlockInfoCode {
    SETBID = 1,
    BLOCKNAME,
    SETRECORDNAME,
}

impl BlockInfoCode {
    pub fn from_u64(v: u64) -> Result<Self> {
        match v {
            1 => Ok(Self::SETBID),
            2 => Ok(Self::BLOCKNAME),
            3 => Ok(Self::SETRECORDNAME),
            _ => Err(anyhow!("'{:?}' is not a balid block info code.", v)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum BlockID {
    MODULE = FIRST_APPLICATION_BLOCKID,
    PARAMATTR,
    PARAMATTR_GROUP,
    CONSTANTS,
    FUNCTION,
    IDENTIFICATION,
    VALUE_SYMTAB,
    METADATA,
    METADATA_ATTACHMENT,
    TYPE_NEW,
    USELIST,
    MODULE_STRTAB,
    GLOBALVAL_SUMMARY,
    OPERAND_BUNDLE_TAGS,
    METADATA_KIND,
    STRTAB,
    FULL_LTO_GLOBALVAL_SUMMARY,
    SYMTAB,
    SYNC_SCOPE_NAMES,
}

impl BlockID {
    pub fn from_u64(v: u64) -> Self {
        match v {
            FIRST_APPLICATION_BLOCKID => Self::MODULE,
            9 => Self::PARAMATTR,
            10 => Self::PARAMATTR_GROUP,
            11 => Self::CONSTANTS,
            12 => Self::FUNCTION,
            13 => Self::IDENTIFICATION,
            14 => Self::VALUE_SYMTAB,
            15 => Self::METADATA,
            16 => Self::METADATA_ATTACHMENT,
            17 => Self::TYPE_NEW,
            18 => Self::USELIST,
            19 => Self::MODULE_STRTAB,
            20 => Self::GLOBALVAL_SUMMARY,
            21 => Self::OPERAND_BUNDLE_TAGS,
            22 => Self::METADATA_KIND,
            23 => Self::STRTAB,
            24 => Self::FULL_LTO_GLOBALVAL_SUMMARY,
            25 => Self::SYMTAB,
            26 => Self::SYNC_SCOPE_NAMES,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum IdentificationCode {
    STRING = 1,
    EPOCH,
}

impl IdentificationCode {
    pub fn from_u64(v: u64) -> Self {
        match v {
            1 => Self::STRING,
            2 => Self::EPOCH,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum ModuleCode {
    VERSION = 1,
    TRIPLE,
    DATALAYOUT,
    ASM,
    SECTIONNAME,

    DEPLIB,

    GLOBALVAR,

    FUNCTION,

    ALIAS_OLD,

    GCNAME = 11,
    COMDAT,

    VSTOFFSET,

    ALIAS,

    METADATA_VALUS_UNUSED,

    SOURCE_FILENAME,

    HASH,

    IFUNC,
}

impl ModuleCode {
    pub fn from_u64(v: u64) -> Self {
        match v {
            1 => Self::VERSION,
            2 => Self::TRIPLE,
            3 => Self::DATALAYOUT,
            4 => Self::ASM,
            5 => Self::SECTIONNAME,
            6 => Self::DEPLIB,
            7 => Self::GLOBALVAR,
            8 => Self::FUNCTION,
            9 => Self::ALIAS_OLD,
            // 10 => doesn't exist afaik.
            11 => Self::GCNAME,
            12 => Self::COMDAT,
            13 => Self::VSTOFFSET,
            14 => Self::ALIAS,
            15 => Self::METADATA_VALUS_UNUSED,
            16 => Self::SOURCE_FILENAME,
            17 => Self::HASH,
            18 => Self::IFUNC,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum TypeCode {
    NUMENTRY = 1,
    VOID,
    FLOAT,
    DOUBLE,
    LABEL,
    OPAQUE,
    INTEGER,
    POINTER,
    FUNCTION_OLD,
    HALF,
    ARRAY,
    VECTOR,
    X86_FP80,
    FP128,
    PPC_FP128,
    METADATA,
    X86_MMX,
    STRUCT_ANON,
    STRUCT_NAME,
    STRUCT_NAMED,
    FUNCTION,
    TOKEN,
    BFLOAT,
    X86_AMX,
    OPAQUE_POINTER,
    TARGET_TYPE,
}

impl TypeCode {
    pub fn from_u64(v: u64) -> Self {
        match v {
            1 => Self::NUMENTRY,
            2 => Self::VOID,
            3 => Self::FLOAT,
            4 => Self::DOUBLE,
            5 => Self::LABEL,
            6 => Self::OPAQUE,
            7 => Self::INTEGER,
            8 => Self::POINTER,
            9 => Self::FUNCTION_OLD,
            10 => Self::HALF,
            11 => Self::ARRAY,
            12 => Self::VECTOR,
            13 => Self::X86_FP80,
            14 => Self::FP128,
            15 => Self::PPC_FP128,
            16 => Self::METADATA,
            17 => Self::X86_MMX,
            18 => Self::STRUCT_ANON,
            19 => Self::STRUCT_NAME,
            20 => Self::STRUCT_NAMED,
            21 => Self::FUNCTION,
            22 => Self::TOKEN,
            23 => Self::BFLOAT,
            24 => Self::X86_AMX,
            25 => Self::OPAQUE_POINTER,
            26 => Self::TARGET_TYPE,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum AttributeKindCode {
    ALIGNMENT = 1,
    ALWAYS_INLINE,
    BY_VAL,
    INLINE_HINT,
    IN_REG,
    MIN_SIZE,
    NAKED,
    NEST,
    NO_ALIAS,
    NO_BUILTIN,
    NO_CAPTURE,
    NO_DUPLICATE,
    NO_IMPLICIT_FLOAT,
    NO_INLINE,
    NON_LAZY_BIND,
    NO_RED_ZONE,
    NO_RETURN,
    NO_UNWIND,
    OPTIMIZE_FOR_SIZE,
    READ_NONE,
    READ_ONLY,
    RETURNED,
    RETURNS_TWICE,
    S_EXT,
    STACK_ALIGNMENT,
    STACK_PROTECT,
    STACK_PROTECT_REQ,
    STACK_PROTECT_STRONG,
    STRUCT_RET,
    SANITIZE_ADDRESS,
    SANITIZE_THREAD,
    SANITIZE_MEMORY,
    UW_TABLE,
    Z_EXT,
    BUILTIN,
    COLD,
    OPTIMIZE_NONE,
    IN_ALLOCA,
    NON_NULL,
    JUMP_TABLE,
    DEREFERENCEABLE,
    DEREFERENCEABLE_OR_NULL,
    CONVERGENT,
    SAFESTACK,
    ARGMEMONLY,
    SWIFT_SELF,
    SWIFT_ERROR,
    NO_RECURSE,
    INACCESSIBLEMEM_ONLY,
    INACCESSIBLEMEM_OR_ARGMEMONLY,
    ALLOC_SIZE,
    WRITEONLY,
    SPECULATABLE,
    STRICT_FP,
    SANITIZE_HWADDRESS,
    NOCF_CHECK,
    OPT_FOR_FUZZING,
    SHADOWCALLSTACK,
    SPECULATIVE_LOAD_HARDENING,
    IMMARG,
    WILLRETURN,
    NOFREE,
    NOSYNC,
    SANITIZE_MEMTAG,
    PREALLOCATED,
    NO_MERGE,
    NULL_POINTER_IS_VALID,
    NOUNDEF,
    BYREF,
    MUSTPROGRESS,
    NO_CALLBACK,
    HOT,
    NO_PROFILE,
    VSCALE_RANGE,
    SWIFT_ASYNC,
    NO_SANITIZE_COVERAGE,
    ELEMENTTYPE,
    DISABLE_SANITIZER_INSTRUMENTATION,
    NO_SANITIZE_BOUNDS,
    ALLOC_ALIGN,
    ALLOCATED_POINTER,
    ALLOC_KIND,
    PRESPLIT_COROUTINE,
    FNRETTHUNK_EXTERN,
    SKIP_PROFILE,
    MEMORY,
    NOFPCLASS,
    OPTIMIZE_FOR_DEBUGGING,
    WRITABLE,
    CORO_ONLY_DESTROY_WHEN_COMPLETE,
    DEAD_ON_UNWIND,
    RANGE,
    SANITIZE_NUMERICAL_STABILITY,
    INITIALIZES,
    HYBRID_PATCHABLE,
    SANITIZE_REALTIME,
    SANITIZE_REALTIME_BLOCKING,
    CORO_ELIDE_SAFE,
    NO_EXT,
    NO_DIVERGENCE_SOURCE,
    SANITIZE_TYPE,
    CAPTURES,
    DEAD_ON_RETURN,
}

impl AttributeKindCode {
    pub fn from_u64(v: u64) -> Self {
        match v {
            1 => Self::ALIGNMENT,
            2 => Self::ALWAYS_INLINE,
            3 => Self::BY_VAL,
            4 => Self::INLINE_HINT,
            5 => Self::IN_REG,
            6 => Self::MIN_SIZE,
            7 => Self::NAKED,
            8 => Self::NEST,
            9 => Self::NO_ALIAS,
            10 => Self::NO_BUILTIN,
            11 => Self::NO_CAPTURE,
            12 => Self::NO_DUPLICATE,
            13 => Self::NO_IMPLICIT_FLOAT,
            14 => Self::NO_INLINE,
            15 => Self::NON_LAZY_BIND,
            16 => Self::NO_RED_ZONE,
            17 => Self::NO_RETURN,
            18 => Self::NO_UNWIND,
            19 => Self::OPTIMIZE_FOR_SIZE,
            20 => Self::READ_NONE,
            21 => Self::READ_ONLY,
            22 => Self::RETURNED,
            23 => Self::RETURNS_TWICE,
            24 => Self::S_EXT,
            25 => Self::STACK_ALIGNMENT,
            26 => Self::STACK_PROTECT,
            27 => Self::STACK_PROTECT_REQ,
            28 => Self::STACK_PROTECT_STRONG,
            29 => Self::STRUCT_RET,
            30 => Self::SANITIZE_ADDRESS,
            31 => Self::SANITIZE_THREAD,
            32 => Self::SANITIZE_MEMORY,
            33 => Self::UW_TABLE,
            34 => Self::Z_EXT,
            35 => Self::BUILTIN,
            36 => Self::COLD,
            37 => Self::OPTIMIZE_NONE,
            38 => Self::IN_ALLOCA,
            39 => Self::NON_NULL,
            40 => Self::JUMP_TABLE,
            41 => Self::DEREFERENCEABLE,
            42 => Self::DEREFERENCEABLE_OR_NULL,
            43 => Self::CONVERGENT,
            44 => Self::SAFESTACK,
            45 => Self::ARGMEMONLY,
            46 => Self::SWIFT_SELF,
            47 => Self::SWIFT_ERROR,
            48 => Self::NO_RECURSE,
            49 => Self::INACCESSIBLEMEM_ONLY,
            50 => Self::INACCESSIBLEMEM_OR_ARGMEMONLY,
            51 => Self::ALLOC_SIZE,
            52 => Self::WRITEONLY,
            53 => Self::SPECULATABLE,
            54 => Self::STRICT_FP,
            55 => Self::SANITIZE_HWADDRESS,
            56 => Self::NOCF_CHECK,
            57 => Self::OPT_FOR_FUZZING,
            58 => Self::SHADOWCALLSTACK,
            59 => Self::SPECULATIVE_LOAD_HARDENING,
            60 => Self::IMMARG,
            61 => Self::WILLRETURN,
            62 => Self::NOFREE,
            63 => Self::NOSYNC,
            64 => Self::SANITIZE_MEMTAG,
            65 => Self::PREALLOCATED,
            66 => Self::NO_MERGE,
            67 => Self::NULL_POINTER_IS_VALID,
            68 => Self::NOUNDEF,
            69 => Self::BYREF,
            70 => Self::MUSTPROGRESS,
            71 => Self::NO_CALLBACK,
            72 => Self::HOT,
            73 => Self::NO_PROFILE,
            74 => Self::VSCALE_RANGE,
            75 => Self::SWIFT_ASYNC,
            76 => Self::NO_SANITIZE_COVERAGE,
            77 => Self::ELEMENTTYPE,
            78 => Self::DISABLE_SANITIZER_INSTRUMENTATION,
            79 => Self::NO_SANITIZE_BOUNDS,
            80 => Self::ALLOC_ALIGN,
            81 => Self::ALLOCATED_POINTER,
            82 => Self::ALLOC_KIND,
            83 => Self::PRESPLIT_COROUTINE,
            84 => Self::FNRETTHUNK_EXTERN,
            85 => Self::SKIP_PROFILE,
            86 => Self::MEMORY,
            87 => Self::NOFPCLASS,
            88 => Self::OPTIMIZE_FOR_DEBUGGING,
            89 => Self::WRITABLE,
            90 => Self::CORO_ONLY_DESTROY_WHEN_COMPLETE,
            91 => Self::DEAD_ON_UNWIND,
            92 => Self::RANGE,
            93 => Self::SANITIZE_NUMERICAL_STABILITY,
            94 => Self::INITIALIZES,
            95 => Self::HYBRID_PATCHABLE,
            96 => Self::SANITIZE_REALTIME,
            97 => Self::SANITIZE_REALTIME_BLOCKING,
            98 => Self::CORO_ELIDE_SAFE,
            99 => Self::NO_EXT,
            100 => Self::NO_DIVERGENCE_SOURCE,
            101 => Self::SANITIZE_TYPE,
            102 => Self::CAPTURES,
            103 => Self::DEAD_ON_RETURN,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum AttributeCode {
    ENTRY_OLD = 1,
    ENTRY,
    GRP_CODE_ENTRY,
}

impl AttributeCode {
    pub fn from_u64(v: u64) -> Self {
        match v {
            1 => Self::ENTRY_OLD,
            2 => Self::ENTRY,
            3 => Self::GRP_CODE_ENTRY,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum ConstantsCode {
    SETTYPE = 1,        // SETTYPE:       [typeid]
    NULL = 2,           // NULL
    UNDEF = 3,          // UNDEF
    INTEGER = 4,        // INTEGER:       [intval]
    WIDE_INTEGER = 5,   // WIDE_INTEGER:  [n x intval]
    FLOAT = 6,          // FLOAT:         [fpval]
    AGGREGATE = 7,      // AGGREGATE:     [n x value number]
    STRING = 8,         // STRING:        [values]
    CSTRING = 9,        // CSTRING:       [values]
    CE_BINOP = 10,      // CE_BINOP:      [opcode, opval, opval]
    CE_CAST = 11,       // CE_CAST:       [opcode, opty, opval]
    CE_GEP_OLD = 12,    // CE_GEP:        [n x operands]
    CE_SELECT = 13,     // CE_SELECT:     [opval, opval, opval]
    CE_EXTRACTELT = 14, // CE_EXTRACTELT: [opty, opval, opval]
    CE_INSERTELT = 15,  // CE_INSERTELT:  [opval, opval, opval]
    CE_SHUFFLEVEC = 16, // CE_SHUFFLEVEC: [opval, opval, opval]
    CE_CMP = 17,        // CE_CMP:        [opty, opval, opval, pred]
    INLINEASM_OLD = 18, // INLINEASM:     [sideeffect|alignstack,
    //                 asmstr,conststr]
    CE_SHUFVEC_EX = 19,   // SHUFVEC_EX:    [opty, opval, opval, opval]
    CE_INBOUNDS_GEP = 20, // INBOUNDS_GEP:  [n x operands]
    BLOCKADDRESS = 21,    // CST_CODE_BLOCKADDRESS [fnty, fnval, bb#]
    DATA = 22,            // DATA:          [n x elements]
    INLINEASM_OLD2 = 23,  // INLINEASM:     [sideeffect|alignstack|
    //                 asmdialect,asmstr,conststr]
    CE_GEP_WITH_INRANGE_INDEX_OLD = 24, //  [opty, flags, n x operands]
    CE_UNOP = 25,                       // CE_UNOP:      [opcode, opval]
    POISON = 26,                        // POISON
    DSO_LOCAL_EQUIVALENT = 27,          // DSO_LOCAL_EQUIVALENT [gvty, gv]
    INLINEASM_OLD3 = 28,                // INLINEASM:     [sideeffect|alignstack|
    //                 asmdialect|unwind,
    //                 asmstr,conststr]
    NO_CFI_VALUE = 29, // NO_CFI [ fty, f ]
    INLINEASM = 30,    // INLINEASM:     [fnty,
    //                 sideeffect|alignstack|
    //                 asmdialect|unwind,
    //                 asmstr,conststr]
    CE_GEP_WITH_INRANGE = 31, // [opty, flags, range, n x operands]
    CE_GEP = 32,              // [opty, flags, n x operands]
    PTRAUTH = 33,             // [ptr, key, disc, addrdisc]
}

impl ConstantsCode {
    pub fn from_u64(v: u64) -> Self {
        match v {
            1 => Self::SETTYPE,
            2 => Self::NULL,
            3 => Self::UNDEF,
            4 => Self::INTEGER,
            5 => Self::WIDE_INTEGER,
            6 => Self::FLOAT,
            7 => Self::AGGREGATE,
            8 => Self::STRING,
            9 => Self::CSTRING,
            10 => Self::CE_BINOP,
            11 => Self::CE_CAST,
            12 => Self::CE_GEP_OLD,
            13 => Self::CE_SELECT,
            14 => Self::CE_EXTRACTELT,
            15 => Self::CE_INSERTELT,
            16 => Self::CE_SHUFFLEVEC,
            17 => Self::CE_CMP,
            18 => Self::INLINEASM_OLD,
            19 => Self::CE_SHUFVEC_EX,
            20 => Self::CE_INBOUNDS_GEP,
            21 => Self::BLOCKADDRESS,
            22 => Self::DATA,
            23 => Self::INLINEASM_OLD2,
            24 => Self::CE_GEP_WITH_INRANGE_INDEX_OLD,
            25 => Self::CE_UNOP,
            26 => Self::POISON,
            27 => Self::DSO_LOCAL_EQUIVALENT,
            28 => Self::INLINEASM_OLD3,
            29 => Self::NO_CFI_VALUE,
            30 => Self::INLINEASM,
            31 => Self::CE_GEP_WITH_INRANGE,
            32 => Self::CE_GEP,
            33 => Self::PTRAUTH,
            _ => unimplemented!(),
        }
    }
}
