use anyhow::{Result, anyhow};
use bitflags::bitflags;

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

#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum MetadataCodes {
    STRING_OLD = 1,              // MDSTRING:      [values]
    VALUE = 2,                   // VALUE:         [type num, value num]
    NODE = 3,                    // NODE:          [n x md num]
    NAME = 4,                    // STRING:        [values]
    DISTINCT_NODE = 5,           // DISTINCT_NODE: [n x md num]
    KIND = 6,                    // [n x [id, name]]
    LOCATION = 7,                // [distinct, line, col, scope, inlined-at?]
    OLD_NODE = 8,                // OLD_NODE:      [n x (type num, value num)]
    OLD_FN_NODE = 9,             // OLD_FN_NODE:   [n x (type num, value num)]
    NAMED_NODE = 10,             // NAMED_NODE:    [n x mdnodes]
    ATTACHMENT = 11,             // [m x [value, [n x [id, mdnode]]]
    GENERIC_DEBUG = 12,          // [distinct, tag, vers, header, n x md num]
    SUBRANGE = 13,               // [distinct, count, lo]
    ENUMERATOR = 14,             // [isUnsigned|distinct, value, name]
    BASIC_TYPE = 15,             // [distinct, tag, name, size, align, enc]
    FILE = 16,                   // [distinct, filename, directory, checksumkind, checksum]
    DERIVED_TYPE = 17,           // [distinct, ...]
    COMPOSITE_TYPE = 18,         // [distinct, ...]
    SUBROUTINE_TYPE = 19,        // [distinct, flags, types, cc]
    COMPILE_UNIT = 20,           // [distinct, ...]
    SUBPROGRAM = 21,             // [distinct, ...]
    LEXICAL_BLOCK = 22,          // [distinct, scope, file, line, column]
    LEXICAL_BLOCK_FILE = 23,     //[distinct, scope, file, discriminator]
    NAMESPACE = 24,              // [distinct, scope, file, name, line, exportSymbols]
    TEMPLATE_TYPE = 25,          // [distinct, scope, name, type, ...]
    TEMPLATE_VALUE = 26,         // [distinct, scope, name, type, value, ...]
    GLOBAL_VAR = 27,             // [distinct, ...]
    LOCAL_VAR = 28,              // [distinct, ...]
    EXPRESSION = 29,             // [distinct, n x element]
    OBJC_PROPERTY = 30,          // [distinct, name, file, line, ...]
    IMPORTED_ENTITY = 31,        // [distinct, tag, scope, entity, line, name]
    MODULE = 32,                 // [distinct, scope, name, ...]
    MACRO = 33,                  // [distinct, macinfo, line, name, value]
    MACRO_FILE = 34,             // [distinct, macinfo, line, file, ...]
    STRINGS = 35,                // [count, offset] blob([lengths][chars])
    GLOBAL_DECL_ATTACHMENT = 36, // [valueid, n x [id, mdnode]]
    GLOBAL_VAR_EXPR = 37,        // [distinct, var, expr]
    INDEX_OFFSET = 38,           // [offset]
    INDEX = 39,                  // [bitpos]
    LABEL = 40,                  // [distinct, scope, name, file, line]
    STRING_TYPE = 41,            // [distinct, name, size, align,...]
    // Codes 42 and 43 are reserved for support for Fortran array specific debug
    // info.
    FORTRAN_RESERVED_1 = 42,
    FORTRAN_RESERVED_2 = 43,

    COMMON_BLOCK = 44,     // [distinct, scope, name, variable,...]
    GENERIC_SUBRANGE = 45, // [distinct, count, lo, up, stride]
    ARG_LIST = 46,         // [n x [type num, value num]]
    ASSIGN_ID = 47,        // [distinct, ...]
}

impl MetadataCodes {
    pub fn from_u64(v: u64) -> Self {
        match v {
            1 => Self::STRING_OLD,
            2 => Self::VALUE,
            3 => Self::NODE,
            4 => Self::NAME,
            5 => Self::DISTINCT_NODE,
            6 => Self::KIND,
            7 => Self::LOCATION,
            8 => Self::OLD_NODE,
            9 => Self::OLD_FN_NODE,
            10 => Self::NAMED_NODE,
            11 => Self::ATTACHMENT,
            12 => Self::GENERIC_DEBUG,
            13 => Self::SUBRANGE,
            14 => Self::ENUMERATOR,
            15 => Self::BASIC_TYPE,
            16 => Self::FILE,
            17 => Self::DERIVED_TYPE,
            18 => Self::COMPOSITE_TYPE,
            19 => Self::SUBROUTINE_TYPE,
            20 => Self::COMPILE_UNIT,
            21 => Self::SUBPROGRAM,
            22 => Self::LEXICAL_BLOCK,
            23 => Self::LEXICAL_BLOCK_FILE,
            24 => Self::NAMESPACE,
            25 => Self::TEMPLATE_TYPE,
            26 => Self::TEMPLATE_VALUE,
            27 => Self::GLOBAL_VAR,
            28 => Self::LOCAL_VAR,
            29 => Self::EXPRESSION,
            30 => Self::OBJC_PROPERTY,
            31 => Self::IMPORTED_ENTITY,
            32 => Self::MODULE,
            33 => Self::MACRO,
            34 => Self::MACRO_FILE,
            35 => Self::STRINGS,
            36 => Self::GLOBAL_DECL_ATTACHMENT,
            37 => Self::GLOBAL_VAR_EXPR,
            38 => Self::INDEX_OFFSET,
            39 => Self::INDEX,
            40 => Self::LABEL,
            41 => Self::STRING_TYPE,
            42 => Self::FORTRAN_RESERVED_1,
            43 => Self::FORTRAN_RESERVED_2,
            44 => Self::COMMON_BLOCK,
            45 => Self::GENERIC_SUBRANGE,
            46 => Self::ARG_LIST,
            47 => Self::ASSIGN_ID,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum FunctionCodes {
    DECLAREBLOCKS = 1, // DECLAREBLOCKS: [n]

    INST_BINOP = 2,      // BINOP:      [opcode, ty, opval, opval]
    INST_CAST = 3,       // CAST:       [opcode, ty, opty, opval]
    INST_GEP_OLD = 4,    // GEP:        [n x operands]
    INST_SELECT = 5,     // SELECT:     [ty, opval, opval, opval]
    INST_EXTRACTELT = 6, // EXTRACTELT: [opty, opval, opval]
    INST_INSERTELT = 7,  // INSERTELT:  [ty, opval, opval, opval]
    INST_SHUFFLEVEC = 8, // SHUFFLEVEC: [ty, opval, opval, opval]
    INST_CMP = 9,        // CMP:        [opty, opval, opval, pred]

    INST_RET = 10,    // RET:        [opty,opval<both optional>]
    INST_BR = 11,     // BR:         [bb#, bb#, cond] or [bb#]
    INST_SWITCH = 12, // SWITCH:     [opty, op0, op1, ...]
    INST_INVOKE = 13, // INVOKE:     [attr, fnty, op0,op1, ...]
    // 14 is unused.
    INST_UNREACHABLE = 15, // UNREACHABLE

    INST_PHI = 16, // PHI:        [ty, val0,bb0, ...]
    // 17 is unused.
    // 18 is unused.
    INST_ALLOCA = 19, // ALLOCA:     [instty, opty, op, align]
    INST_LOAD = 20,   // LOAD:       [opty, op, align, vol]
    // 21 is unused.
    // 22 is unused.
    INST_VAARG = 23, // VAARG:      [valistty, valist, instty]
    // This store code encodes the pointer type, rather than the value type
    // this is so information only available in the pointer type (e.g. address
    // spaces) is retained.
    INST_STORE_OLD = 24, // STORE:      [ptrty,ptr,val, align, vol]
    // 25 is unused.
    INST_EXTRACTVAL = 26, // EXTRACTVAL: [n x operands]
    INST_INSERTVAL = 27,  // INSERTVAL:  [n x operands]
    // fcmp/icmp returning Int1TY or vector of Int1Ty. Same as CMP, exists to
    // support legacy vicmp/vfcmp instructions.
    INST_CMP2 = 28, // CMP2:       [opty, opval, opval, pred]
    // new select on i1 or [N x i1]
    INST_VSELECT = 29,          // VSELECT:    [ty,opval,opval,predty,pred]
    INST_INBOUNDS_GEP_OLD = 30, // INBOUNDS_GEP: [n x operands]
    INST_INDIRECTBR = 31,       // INDIRECTBR: [opty, op0, op1, ...]
    // 32 is unused.
    DEBUG_LOC_AGAIN = 33, // DEBUG_LOC_AGAIN

    INST_CALL = 34, // CALL:    [attr, cc, fnty, fnid, args...]

    DEBUG_LOC = 35,        // DEBUG_LOC:  [Line,Col,ScopeVal, IAVal]
    INST_FENCE = 36,       // FENCE: [ordering, synchscope]
    INST_CMPXCHG_OLD = 37, // CMPXCHG: [ptrty, ptr, cmp, val, vol,
    //            ordering, synchscope,
    //            failure_ordering?, weak?]
    INST_ATOMICRMW_OLD = 38, // ATOMICRMW: [ptrty,ptr,val, operation,
    //             align, vol,
    //             ordering, synchscope]
    INST_RESUME = 39,         // RESUME:     [opval]
    INST_LANDINGPAD_OLD = 40, // LANDINGPAD: [ty,val,val,num,id0,val0...]
    INST_LOADATOMIC = 41,     // LOAD: [opty, op, align, vol,
    //        ordering, synchscope]
    INST_STOREATOMIC_OLD = 42, // STORE: [ptrty,ptr,val, align, vol
    //         ordering, synchscope]
    INST_GEP = 43,         // GEP:  [inbounds, n x operands]
    INST_STORE = 44,       // STORE: [ptrty,ptr,valty,val, align, vol]
    INST_STOREATOMIC = 45, // STORE: [ptrty,ptr,val, align, vol
    INST_CMPXCHG = 46,     // CMPXCHG: [ptrty, ptr, cmp, val, vol,
    //           success_ordering, synchscope,
    //           failure_ordering, weak]
    INST_LANDINGPAD = 47,  // LANDINGPAD: [ty,val,num,id0,val0...]
    INST_CLEANUPRET = 48,  // CLEANUPRET: [val] or [val,bb#]
    INST_CATCHRET = 49,    // CATCHRET: [val,bb#]
    INST_CATCHPAD = 50,    // CATCHPAD: [bb#,bb#,num,args...]
    INST_CLEANUPPAD = 51,  // CLEANUPPAD: [num,args...]
    INST_CATCHSWITCH = 52, // CATCHSWITCH: [num,args...] or [num,args...,bb]
    // 53 is unused.
    // 54 is unused.
    OPERAND_BUNDLE = 55, // OPERAND_BUNDLE: [tag#, value...]
    INST_UNOP = 56,      // UNOP:       [opcode, ty, opval]
    INST_CALLBR = 57,    // CALLBR:     [attr, cc, norm, transfs,
    //              fnty, fnid, args...]
    INST_FREEZE = 58,    // FREEZE: [opty, opval]
    INST_ATOMICRMW = 59, // ATOMICRMW: [ptrty, ptr, valty, val,
    //             operation, align, vol,
    //             ordering, synchscope]
    BLOCKADDR_USERS = 60, // BLOCKADDR_USERS: [value...]

    DEBUG_RECORD_VALUE = 61, // [DILocation, DILocalVariable, DIExpression, ValueAsMetadata]
    DEBUG_RECORD_DECLARE = 62, // [DILocation, DILocalVariable, DIExpression, ValueAsMetadata]
    DEBUG_RECORD_ASSIGN = 63, // [DILocation, DILocalVariable, DIExpression, ValueAsMetadata,
    //  DIAssignID, DIExpression (addr), ValueAsMetadata (addr)]
    DEBUG_RECORD_VALUE_SIMPLE = 64, // [DILocation, DILocalVariable, DIExpression, Value]
    DEBUG_RECORD_LABEL = 65,        // [DILocation, DILabel]
}

impl FunctionCodes {
    pub fn from_u64(v: u64) -> Self {
        match v {
            1 => Self::DECLAREBLOCKS,
            2 => Self::INST_BINOP,
            3 => Self::INST_CAST,
            4 => Self::INST_GEP_OLD,
            5 => Self::INST_SELECT,
            6 => Self::INST_EXTRACTELT,
            7 => Self::INST_INSERTELT,
            8 => Self::INST_SHUFFLEVEC,
            9 => Self::INST_CMP,
            10 => Self::INST_RET,
            11 => Self::INST_BR,
            12 => Self::INST_SWITCH,
            13 => Self::INST_INVOKE,
            // 14 is Unused.
            15 => Self::INST_UNREACHABLE,
            16 => Self::INST_PHI,
            // 17 is Unused.
            // 18 is Unused.
            19 => Self::INST_ALLOCA,
            20 => Self::INST_LOAD,
            // 21 is Unused.
            // 22 is Unused.
            23 => Self::INST_VAARG,
            24 => Self::INST_STORE_OLD,
            // 25 is Unused.
            26 => Self::INST_EXTRACTVAL,
            27 => Self::INST_INSERTVAL,
            28 => Self::INST_CMP2,
            29 => Self::INST_VSELECT,
            30 => Self::INST_INBOUNDS_GEP_OLD,
            31 => Self::INST_INDIRECTBR,
            // 32 is Unused.
            33 => Self::DEBUG_LOC_AGAIN,
            34 => Self::INST_CALL,
            35 => Self::DEBUG_LOC,
            36 => Self::INST_FENCE,
            37 => Self::INST_CMPXCHG_OLD,
            38 => Self::INST_ATOMICRMW_OLD,
            39 => Self::INST_RESUME,
            40 => Self::INST_LANDINGPAD_OLD,
            41 => Self::INST_LOADATOMIC,
            42 => Self::INST_STOREATOMIC_OLD,
            43 => Self::INST_GEP,
            44 => Self::INST_STORE,
            45 => Self::INST_STOREATOMIC,
            46 => Self::INST_CMPXCHG,
            47 => Self::INST_LANDINGPAD,
            48 => Self::INST_CLEANUPRET,
            49 => Self::INST_CATCHRET,
            50 => Self::INST_CATCHPAD,
            51 => Self::INST_CLEANUPPAD,
            52 => Self::INST_CATCHSWITCH,
            // 53 is Unused.
            // 54 is Unused.
            55 => Self::OPERAND_BUNDLE,
            56 => Self::INST_UNOP,
            57 => Self::INST_CALLBR,
            58 => Self::INST_FREEZE,
            59 => Self::INST_ATOMICRMW,
            60 => Self::BLOCKADDR_USERS,
            61 => Self::DEBUG_RECORD_VALUE,
            62 => Self::DEBUG_RECORD_DECLARE,
            63 => Self::DEBUG_RECORD_ASSIGN,
            64 => Self::DEBUG_RECORD_VALUE_SIMPLE,
            65 => Self::DEBUG_RECORD_LABEL,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum CastOpCode {
    #[default]
    TRUNC = 0,
    ZEXT = 1,
    SEXT = 2,
    FPTOUI = 3,
    FPTOSI = 4,
    UITOFP = 5,
    SITOFP = 6,
    FPTRUNC = 7,
    FPEXT = 8,
    PTRTOINT = 9,
    INTTOPTR = 10,
    BITCAST = 11,
    ADDRSPACECAST = 12,
}

impl CastOpCode {
    pub fn from_u64(v: u64) -> Self {
        match v {
            0 => Self::TRUNC,
            1 => Self::ZEXT,
            2 => Self::SEXT,
            3 => Self::FPTOUI,
            4 => Self::FPTOSI,
            5 => Self::UITOFP,
            6 => Self::SITOFP,
            7 => Self::FPTRUNC,
            8 => Self::FPEXT,
            9 => Self::PTRTOINT,
            10 => Self::INTTOPTR,
            11 => Self::BITCAST,
            12 => Self::ADDRSPACECAST,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum GetElementPtrOptionalFlags {
    #[default]
    INBOUNDS = 0,
    NUSW = 1,
    NUW = 2,
}

bitflags! {
    #[derive(Debug, Default, Clone, PartialEq)]
    pub struct GEPNoWrapFlags: u64 {
        const InBoundsFlag = (1 << 0);
        const NUSWFlag = (1 << 1);
        const NUWFlag = (1 << 2);
    }
}

impl GEPNoWrapFlags {
    pub fn from_u64(v: u64) -> Self {
        let mut nw = Self::default();

        if v & (1 << GetElementPtrOptionalFlags::INBOUNDS as u64) != 0 {
            nw |= Self::InBoundsFlag;
        }
        if v & (1 << GetElementPtrOptionalFlags::NUSW as u64) != 0 {
            nw |= Self::NUSWFlag;
        }
        if v & (1 << GetElementPtrOptionalFlags::NUW as u64) != 0 {
            nw |= Self::NUWFlag;
        }

        nw
    }
}
