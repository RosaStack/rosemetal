use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::llvm_bitcode::{AttributeKindCode, Fields};

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
}

#[derive(Debug, Clone)]
pub enum AIRConstant {}

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
    pub global_variables: Vec<AIRGlobalVariable>,
    pub constants: Vec<AIRConstant>,
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
        let init_id = fields[4];

        todo!()
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
