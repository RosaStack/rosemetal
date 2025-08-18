use std::collections::HashMap;

use crate::llvm_bitcode::AttributeKindCode;

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
pub struct AIRModule {
    pub version: u64,
    pub types: Vec<AIRType>,
    pub attributes: HashMap<u64, AIRAttribute>,
    pub entry_table: HashMap<u64, AIRAttrEntry>,
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
