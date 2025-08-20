pub mod items;

use std::collections::HashMap;

pub use items::*;

use anyhow::{Result, anyhow};

use crate::llvm_bitcode::{
    AttributeCode, AttributeKindCode, Bitstream, Block, BlockID, ConstantsCode, Fields,
    IdentificationCode, ModuleCode, Record, Signature, StreamEntry, TypeCode,
};

pub struct Parser {
    signature: Option<Signature>,
    bitstream: Bitstream,
}

impl Parser {
    pub fn new(inner: Vec<u8>) -> Result<Self> {
        let (signature, bitstream) = Bitstream::from(inner)?;

        Ok(Self {
            signature,
            bitstream,
        })
    }

    pub fn parse_block(&mut self, b: Block) -> Result<AIRItem> {
        match BlockID::from_u64(b.block_id) {
            BlockID::IDENTIFICATION => Ok(AIRItem::IdentificationBlock(
                self.parse_identification_block()?,
            )),
            BlockID::MODULE => Ok(AIRItem::Module(self.parse_module()?)),
            _ => todo!("{:?} not implemented yet.", b),
        }
    }

    pub fn parse_module_record(&mut self, record: Record, result: &mut AIRModule) {
        match ModuleCode::from_u64(record.code) {
            ModuleCode::VERSION => result.version = record.fields[0],
            ModuleCode::TRIPLE => result.triple = Self::parse_string(record.fields),
            ModuleCode::DATALAYOUT => result.data_layout = Self::parse_string(record.fields),
            ModuleCode::SOURCE_FILENAME => {
                result.source_filename = Self::parse_string(record.fields)
            }
            ModuleCode::GLOBALVAR => result.parse_global_variable(record.fields),
            ModuleCode::FUNCTION => result.parse_function_signature(record.fields),
            ModuleCode::VSTOFFSET => result
                .undiscovered_data
                .push(UndiscoveredData::VSTOFFSET(record.fields[0])),
            _ => todo!("{:?} | {:?}", ModuleCode::from_u64(record.code), record),
        }
    }

    pub fn parse_type_entries(&mut self) -> Result<Vec<AIRType>> {
        let mut content = self.bitstream.next();
        let mut result: Vec<AIRType> = vec![];
        let mut last_struct_name = String::new();

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => {
                        return Ok(result);
                    }
                    StreamEntry::Record(record) => match TypeCode::from_u64(record.code) {
                        TypeCode::NUMENTRY => {}
                        TypeCode::FLOAT => result.push(AIRType::Float),
                        TypeCode::VECTOR => result.push(AIRType::Vector(AIRVectorType {
                            size: record.fields[0],
                            element_type: Box::new(result[record.fields[1] as usize].clone()),
                        })),
                        TypeCode::ARRAY => result.push(AIRType::Array(AIRArrayType {
                            size: record.fields[0],
                            element_type: Box::new(result[record.fields[1] as usize].clone()),
                        })),
                        TypeCode::STRUCT_NAME => {
                            last_struct_name = Self::parse_string(record.fields);
                        }
                        TypeCode::STRUCT_NAMED | TypeCode::STRUCT_ANON => {
                            let mut elements: Vec<AIRType> = vec![];

                            let is_packed = record.fields[0] != 0;

                            for i in 1..record.fields.len() {
                                elements.push(result[record.fields[i] as usize].clone());
                            }

                            result.push(AIRType::Struct(AIRStructType {
                                name: last_struct_name.clone(),
                                is_packed,
                                elements,
                            }));

                            last_struct_name.clear();
                        }
                        TypeCode::INTEGER => result.push(AIRType::Integer(record.fields[0])),
                        TypeCode::POINTER => result.push(AIRType::Pointer(
                            record.fields[1],
                            Box::new(result[record.fields[0] as usize].clone()),
                        )),
                        TypeCode::FUNCTION => {
                            let mut params: Vec<AIRType> = vec![];

                            dbg!(&result);

                            for i in 2..record.fields.len() {
                                let i = record.fields[i];
                                params.push(result[i as usize].clone());
                            }

                            result.push(AIRType::Function(AIRFunctionType {
                                vararg: record.fields[0],
                                return_type: Box::new(result[record.fields[1] as usize].clone()),
                                params,
                            }))
                        }
                        TypeCode::METADATA => result.push(AIRType::Metadata),
                        TypeCode::VOID => result.push(AIRType::Void),
                        _ => todo!("{:?}", TypeCode::from_u64(record.code)),
                    },
                    _ => todo!(),
                },
                None => return Ok(result),
            }

            content = self.bitstream.next();
        }
    }

    pub fn parse_attribute(&mut self, record: Record) -> Result<AIRAttribute> {
        if record.code == 0 || record.code > 3 {
            return Err(anyhow!("Invalid code for attribute parsing."));
        }

        let mut result = AIRAttribute::default();
        match AttributeCode::from_u64(record.code) {
            AttributeCode::GRP_CODE_ENTRY => {
                result.id = record.fields[0];
                result.paramidx = record.fields[1];

                let mut count = 2;
                while count < record.fields.len() {
                    match record.fields[count] {
                        0 => {
                            count += 1;
                            let property = AttributeKindCode::from_u64(record.fields[count]);
                            result
                                .properties
                                .push(AIRAttrProperties::WellKnown(property));
                        }
                        3 => {
                            count += 1;
                            let (string, idx) =
                                Self::parse_null_terminated_string(&record.fields, count as u64);

                            count = idx as usize;

                            result
                                .properties
                                .push(AIRAttrProperties::StringAttribute(string));
                        }
                        4 => {
                            count += 1;
                            let (first, idx_one) =
                                Self::parse_null_terminated_string(&record.fields, count as u64);

                            count = idx_one as usize + 1;

                            let (second, idx_two) =
                                Self::parse_null_terminated_string(&record.fields, count as u64);

                            count = idx_two as usize;

                            result
                                .properties
                                .push(AIRAttrProperties::WithStringValue(first, second));
                        }
                        _ => todo!("{:?}", record.fields[count]),
                    }

                    count += 1;
                }

                return Ok(result);
            }
            _ => todo!(),
        }
    }

    pub fn parse_null_terminated_string(fields: &Fields, start_idx: u64) -> (String, u64) {
        let mut result = String::new();

        let mut count = start_idx;
        while count < fields.len() as u64 {
            let character = fields[count as usize];

            if character == 0 {
                break;
            }

            result.push(character as u8 as char);
            count += 1;
        }

        (result, count)
    }

    pub fn parse_attribute_group(&mut self) -> Result<HashMap<u64, AIRAttribute>> {
        let mut content = self.bitstream.next();
        let mut result: HashMap<u64, AIRAttribute> = HashMap::new();

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => {
                        return Ok(result);
                    }
                    StreamEntry::Record(record) => {
                        let property = self.parse_attribute(record)?;
                        result.insert(property.id, property);
                    }

                    _ => todo!(),
                },
                None => return Ok(result),
            }

            content = self.bitstream.next();
        }
    }

    pub fn parse_entry(&mut self, record: Record) -> Result<AIRAttrEntry> {
        match AttributeCode::from_u64(record.code) {
            AttributeCode::ENTRY => {
                return Ok(AIRAttrEntry {
                    groups: record.fields,
                });
            }
            _ => todo!(),
        }
    }

    pub fn parse_entry_table(&mut self) -> Result<HashMap<u64, AIRAttrEntry>> {
        let mut content = self.bitstream.next();
        let mut result: HashMap<u64, AIRAttrEntry> = HashMap::new();

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => {
                        return Ok(result);
                    }
                    StreamEntry::Record(record) => {
                        let entry = self.parse_entry(record)?;
                        result.insert(result.len() as u64 + 1, entry);
                    }

                    _ => todo!(),
                },
                None => return Ok(result),
            }

            content = self.bitstream.next();
        }
    }

    pub fn parse_aggregate(
        &mut self,
        result: &mut AIRModule,
        record: &Record,
        current_type: &AIRType,
    ) -> Result<AIRConstant> {
        let mut contents: Vec<AIRConstant> = vec![];
        for i in &record.fields {
            contents.push(result.constants.get(i).unwrap().clone());
        }

        Ok(AIRConstant {
            ty: current_type.clone(),
            value: AIRConstantValue::Aggregate(contents),
        })
    }

    pub fn parse_constant_data(
        &mut self,
        array_ty: &AIRType,
        fields: Fields,
    ) -> Result<AIRConstantValue> {
        let element_type = match &array_ty {
            AIRType::Vector(v) => &v.element_type,
            AIRType::Array(a) => &a.element_type,
            _ => todo!(),
        };

        let mut contents: Vec<AIRConstantValue> = vec![];
        for i in fields {
            match **element_type {
                AIRType::Float => {
                    let result = f32::from_le_bytes((i as u32).to_le_bytes());
                    contents.push(AIRConstantValue::Float32(result));
                }
                AIRType::Integer(_) => {
                    contents.push(AIRConstantValue::Integer(i));
                }
                _ => todo!("{:?}", **element_type),
            }
        }

        Ok(AIRConstantValue::Array(contents))
    }

    pub fn parse_constants(&mut self, result: &mut AIRModule) -> Result<()> {
        let mut content = self.bitstream.next();
        let mut current_type = AIRType::Void;
        let mut max_id = 0;

        let mut skip_add_one_in_settype = false;
        loop {
            match content {
                Some(ucontent) => match ucontent? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => break,
                    StreamEntry::Record(record) => match ConstantsCode::from_u64(record.code) {
                        ConstantsCode::SETTYPE => {
                            current_type = result.types[record.fields[0] as usize].clone();

                            if skip_add_one_in_settype {
                                content = self.bitstream.next();
                                skip_add_one_in_settype = false;
                                continue;
                            }
                        }
                        ConstantsCode::INTEGER => {
                            let _ = result.constants.insert(
                                max_id,
                                AIRConstant {
                                    ty: current_type.clone(),
                                    value: AIRConstantValue::Integer(record.fields[0]),
                                },
                            );
                        }
                        ConstantsCode::NULL => {
                            let _ = result.constants.insert(
                                max_id,
                                AIRConstant {
                                    ty: current_type.clone(),
                                    value: AIRConstantValue::Null,
                                },
                            );
                        }
                        ConstantsCode::AGGREGATE => {
                            let aggregate = self.parse_aggregate(result, &record, &current_type)?;
                            let _ = result.constants.insert(max_id, aggregate);
                            skip_add_one_in_settype = true;
                        }
                        ConstantsCode::DATA => {
                            let _ = result.constants.insert(
                                max_id,
                                AIRConstant {
                                    ty: current_type.clone(),
                                    value: self
                                        .parse_constant_data(&current_type, record.fields)?,
                                },
                            );
                            skip_add_one_in_settype = true;
                        }
                        _ => todo!("{:#?}", ConstantsCode::from_u64(record.code)),
                    },

                    _ => todo!(),
                },
                None => break,
            }

            max_id += 1;

            content = self.bitstream.next();
        }

        dbg!(&result.constants);

        Ok(())
    }

    pub fn parse_module_sub_block(
        &mut self,
        sub_block: Block,
        result: &mut AIRModule,
    ) -> Result<()> {
        match BlockID::from_u64(sub_block.block_id) {
            BlockID::TYPE_NEW => result.types = self.parse_type_entries()?,
            BlockID::PARAMATTR_GROUP => result.attributes = self.parse_attribute_group()?,
            BlockID::PARAMATTR => result.entry_table = self.parse_entry_table()?,
            BlockID::CONSTANTS => self.parse_constants(result)?,
            BlockID::METADATA_KIND => todo!(),
            _ => todo!("{:?}", BlockID::from_u64(sub_block.block_id)),
        }

        Ok(())
    }

    pub fn parse_module(&mut self) -> Result<AIRModule> {
        let mut content = self.bitstream.next();
        let mut result = AIRModule::default();

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => {
                        return Ok(result);
                    }
                    StreamEntry::SubBlock(sub_block) => {
                        self.parse_module_sub_block(sub_block, &mut result)?;
                    }
                    StreamEntry::Record(record) => {
                        self.parse_module_record(record, &mut result);
                    }
                },
                None => return Ok(result),
            }

            content = self.bitstream.next();
        }
    }

    pub fn parse_string(fields: Fields) -> String {
        let mut result = String::new();

        for i in fields {
            result.push(i as u8 as char);
        }

        result
    }

    pub fn parse_identification_block(&mut self) -> Result<AIRIdentificationBlock> {
        let mut content = self.bitstream.next();
        let mut result = AIRIdentificationBlock::default();

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => {
                        return Ok(result);
                    }
                    StreamEntry::Record(record) => {
                        match IdentificationCode::from_u64(record.code) {
                            IdentificationCode::STRING => {
                                result.string = Self::parse_string(record.fields);
                            }
                            IdentificationCode::EPOCH => {
                                result.epoch = record.fields;
                            }
                        }
                    }
                    _ => todo!(),
                },
                None => return Ok(result),
            }

            content = self.bitstream.next();
        }
    }

    pub fn next(&mut self) -> Result<Option<AIRItem>> {
        match self.bitstream.next() {
            Some(entry) => match entry? {
                StreamEntry::SubBlock(b) => Ok(Some(self.parse_block(b)?)),
                _ => todo!(),
            },
            None => Ok(None),
        }
    }

    pub fn start(&mut self) -> Result<AIRFile> {
        let mut items: Vec<AIRItem> = vec![];
        let mut content = self.next()?;

        loop {
            match &content {
                Some(content) => items.push(content.clone()),
                None => break,
            }

            content = self.next()?;
        }

        Ok(AIRFile { items })
    }
}
