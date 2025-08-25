pub mod items;

use std::{
    cell::{RefCell, UnsafeCell},
    collections::HashMap,
    rc::Rc,
};

pub use items::*;

use anyhow::{Result, anyhow};

use crate::llvm_bitcode::{
    AttributeCode, AttributeKindCode, BitCursor, Bitstream, Block, BlockID, ConstantsCode, Fields,
    FunctionCodes, IdentificationCode, MetadataCodes, ModuleCode, Record, Signature, StreamEntry,
    TypeCode,
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

    pub fn parse_module_record(&mut self, record: Record, result: &mut AIRModule) -> Result<()> {
        match ModuleCode::from_u64(record.code) {
            ModuleCode::VERSION => result.version = record.fields[0],
            ModuleCode::TRIPLE => result.triple = Self::parse_string(record.fields),
            ModuleCode::DATALAYOUT => result.data_layout = Self::parse_string(record.fields),
            ModuleCode::SOURCE_FILENAME => {
                result.source_filename = Self::parse_string(record.fields)
            }
            ModuleCode::GLOBALVAR => result.parse_global_variable(record.fields),
            ModuleCode::FUNCTION => result.parse_function_signature(record.fields)?,
            ModuleCode::VSTOFFSET => result
                .undiscovered_data
                .push(UndiscoveredData::VSTOFFSET(record.fields[0])),
            _ => todo!("{:?} | {:?}", ModuleCode::from_u64(record.code), record),
        }

        Ok(())
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

                            for i in 2..record.fields.len() {
                                let i = record.fields[i];
                                params.push(result[i as usize].clone());
                            }

                            result.push(AIRType::Function(AIRFunctionType {
                                vararg: record.fields[0],
                                return_type: Box::new(result[record.fields[1] as usize].clone()),
                                params,
                            }));
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

    pub fn parse_entry(&mut self, record: Record, module: &mut AIRModule) -> Result<AIRAttrEntry> {
        match AttributeCode::from_u64(record.code) {
            AttributeCode::ENTRY => {
                let mut result: Vec<AIRAttribute> = vec![];

                for i in record.fields {
                    result.push(module.attributes[&i].clone());
                }

                return Ok(AIRAttrEntry { groups: result });
            }
            _ => todo!(),
        }
    }

    pub fn parse_entry_table(
        &mut self,
        module: &mut AIRModule,
    ) -> Result<HashMap<u64, AIRAttrEntry>> {
        let mut content = self.bitstream.next();
        let mut result: HashMap<u64, AIRAttrEntry> = HashMap::new();

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => {
                        return Ok(result);
                    }
                    StreamEntry::Record(record) => {
                        let entry = self.parse_entry(record, module)?;
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
        constants: &mut HashMap<u64, Rc<RefCell<AIRConstant>>>,
        record: &Record,
        current_type: &AIRType,
    ) -> Result<AIRConstant> {
        let mut contents: Vec<Rc<RefCell<AIRConstant>>> = vec![];
        for i in &record.fields {
            let result = constants
                .entry(*i)
                .or_insert(Rc::new(RefCell::new(AIRConstant {
                    ty: AIRType::Void,
                    value: AIRConstantValue::Unresolved(*i),
                })));

            contents.push(result.clone());
        }

        Ok(AIRConstant {
            ty: current_type.clone(),
            value: AIRConstantValue::Aggregate(contents),
        })
    }

    pub fn parse_constant_value_with_type(
        result: &mut AIRModule,
        ty: &AIRType,
        value: u64,
    ) -> AIRConstantValue {
        match ty {
            AIRType::Float => {
                let result = f32::from_le_bytes((value as u32).to_le_bytes());
                return AIRConstantValue::Float32(result);
            }
            AIRType::Integer(_) => {
                return AIRConstantValue::Integer(value);
            }
            AIRType::Array(_) => {
                return result.constants[&value].borrow().value.clone();
            }
            AIRType::Pointer(_, _) => {
                return AIRConstantValue::Pointer(value);
            }
            _ => todo!("{:?}", ty),
        }
    }

    pub fn parse_constant_data(
        &mut self,
        result: &mut AIRModule,
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
            contents.push(Self::parse_constant_value_with_type(
                result,
                &**element_type,
                i,
            ));
        }

        Ok(AIRConstantValue::Array(contents))
    }

    pub fn parse_constants(&mut self, module: &mut AIRModule) -> Result<()> {
        let mut content = self.bitstream.next();
        let mut current_type = AIRType::Void;

        let mut skip_add_one_in_settype = false;
        loop {
            match content {
                Some(ucontent) => {
                    match ucontent? {
                        StreamEntry::EndBlock | StreamEntry::EndOfStream => break,
                        StreamEntry::Record(record) => match ConstantsCode::from_u64(record.code) {
                            ConstantsCode::SETTYPE => {
                                current_type = module.types[record.fields[0] as usize].clone();

                                if skip_add_one_in_settype {
                                    content = self.bitstream.next();
                                    skip_add_one_in_settype = false;
                                    continue;
                                }
                            }
                            ConstantsCode::INTEGER => {
                                let value = module
                                    .constants
                                    .entry(module.max_constants_id)
                                    .or_insert(Rc::new(RefCell::new(AIRConstant {
                                        ty: AIRType::Void,
                                        value: AIRConstantValue::Unresolved(
                                            module.max_constants_id,
                                        ),
                                    })));

                                let mut value = value.borrow_mut();
                                value.ty = current_type.clone();
                                value.value = AIRConstantValue::Integer(record.fields[0]);
                            }
                            ConstantsCode::NULL => {
                                let value = module
                                    .constants
                                    .entry(module.max_constants_id)
                                    .or_insert(Rc::new(RefCell::new(AIRConstant {
                                        ty: AIRType::Void,
                                        value: AIRConstantValue::Unresolved(
                                            module.max_constants_id,
                                        ),
                                    })));

                                let mut value = value.borrow_mut();
                                value.ty = current_type.clone();
                                value.value = AIRConstantValue::Null;
                            }
                            ConstantsCode::UNDEF => {
                                let value = module
                                    .constants
                                    .entry(module.max_constants_id)
                                    .or_insert(Rc::new(RefCell::new(AIRConstant {
                                        ty: AIRType::Void,
                                        value: AIRConstantValue::Unresolved(
                                            module.max_constants_id,
                                        ),
                                    })));

                                let mut value = value.borrow_mut();
                                value.ty = current_type.clone();
                                value.value = AIRConstantValue::Undefined;
                            }
                            ConstantsCode::AGGREGATE => {
                                let aggregate = self.parse_aggregate(
                                    &mut module.constants,
                                    &record,
                                    &current_type,
                                )?;

                                let value = module
                                    .constants
                                    .entry(module.max_constants_id)
                                    .or_insert(Rc::new(RefCell::new(AIRConstant {
                                        ty: AIRType::Void,
                                        value: AIRConstantValue::Unresolved(
                                            module.max_constants_id,
                                        ),
                                    })));

                                *value.borrow_mut() = aggregate;
                                skip_add_one_in_settype = true;
                            }
                            ConstantsCode::DATA => {
                                let data = AIRConstant {
                                    ty: current_type.clone(),
                                    value: self.parse_constant_data(
                                        module,
                                        &current_type,
                                        record.fields,
                                    )?,
                                };

                                let value = module
                                    .constants
                                    .entry(module.max_constants_id)
                                    .or_insert(Rc::new(RefCell::new(AIRConstant {
                                        ty: AIRType::Void,
                                        value: AIRConstantValue::Unresolved(
                                            module.max_constants_id,
                                        ),
                                    })));
                                *value.borrow_mut() = data;
                                skip_add_one_in_settype = true;
                            }
                            ConstantsCode::POISON => {
                                let value = module
                                    .constants
                                    .entry(module.max_constants_id)
                                    .or_insert(Rc::new(RefCell::new(AIRConstant {
                                        ty: AIRType::Void,
                                        value: AIRConstantValue::Unresolved(
                                            module.max_constants_id,
                                        ),
                                    })));

                                let mut value = value.borrow_mut();
                                value.ty = current_type.clone();
                                value.value = AIRConstantValue::Poison;
                            }
                            ConstantsCode::FLOAT => {
                                let value = module
                                    .constants
                                    .entry(module.max_constants_id)
                                    .or_insert(Rc::new(RefCell::new(AIRConstant {
                                        ty: AIRType::Void,
                                        value: AIRConstantValue::Unresolved(
                                            module.max_constants_id,
                                        ),
                                    })));

                                let mut value = value.borrow_mut();
                                value.ty = current_type.clone();
                                value.value = AIRConstantValue::Float32(f32::from_le_bytes(
                                    (record.fields[0] as u32).to_le_bytes(),
                                ));
                            }
                            _ => todo!("{:#?}", ConstantsCode::from_u64(record.code)),
                        },

                        _ => todo!(),
                    }
                }
                None => break,
            }

            module.max_constants_id += 1;

            content = self.bitstream.next();
        }

        Ok(())
    }

    pub fn parse_metadata_kind_block(&mut self, result: &mut AIRModule) -> Result<()> {
        let mut content = self.bitstream.next();

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => return Ok(()),
                    StreamEntry::Record(record) => match MetadataCodes::from_u64(record.code) {
                        MetadataCodes::KIND => {
                            let _ = result.metadata_kind_table.insert(
                                record.fields[0],
                                AIRMetadataKind {
                                    id: record.fields[0],
                                    name: Self::parse_string(record.fields[1..].to_vec()),
                                },
                            );
                        }
                        _ => todo!(),
                    },
                    _ => todo!(),
                },
                None => return Ok(()),
            }

            content = self.bitstream.next();
        }
    }

    pub fn parse_metadata_strings(fields: Fields) -> Result<Vec<String>> {
        let mut result: Vec<String> = vec![];

        let length = fields[0];
        let offset = fields[1];

        let mut lengths_vec: Vec<u64> = vec![];

        let mut count = 0;
        while count != length {
            lengths_vec.push(fields[count as usize + 2]);
            count += 1;
        }

        let length_stream = lengths_vec.iter().map(|x| *x as u8).collect::<Vec<_>>();

        let mut cursor = BitCursor::new(length_stream);

        let mut count = 0;
        let mut pointer = 0;
        while count != length {
            let size = cursor.read_vbr(6)?;
            let end = pointer + size;
            let mut string = String::new();
            while pointer != end {
                string.push((fields[offset as usize + 2 + pointer as usize]) as u8 as char);
                pointer += 1;
            }
            result.push(string);
            count += 1;
        }

        Ok(result)
    }

    pub fn parse_metadata_block(&mut self, result: &mut AIRModule) -> Result<()> {
        let mut content = self.bitstream.next();
        let mut current_name = String::new();
        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => return Ok(()),
                    StreamEntry::Record(record) => match MetadataCodes::from_u64(record.code) {
                        MetadataCodes::STRINGS => {
                            result.metadata_strings = Self::parse_metadata_strings(record.fields)?
                        }
                        MetadataCodes::INDEX_OFFSET => result
                            .undiscovered_data
                            .push(UndiscoveredData::INDEX_OFFSET(record.fields[0])),
                        MetadataCodes::VALUE => {
                            let ty = result.types[record.fields[0] as usize].clone();
                            let constant = AIRMetadataConstant::Value(
                                Self::parse_constant_value_with_type(result, &ty, record.fields[1]),
                            );
                            let _ = result
                                .metadata_constants
                                .insert(result.metadata_constants.len() as u64 + 1, constant);
                        }
                        MetadataCodes::NODE | MetadataCodes::NAMED_NODE => {
                            let _ = result.metadata_constants.insert(
                                result.metadata_constants.len() as u64 + 1,
                                AIRMetadataConstant::Node(current_name.clone(), record.fields),
                            );
                            current_name.clear();
                        }
                        MetadataCodes::INDEX => {
                            // Skip. We don't need this data (for now).
                        }
                        MetadataCodes::NAME => {
                            current_name = Self::parse_string(record.fields);
                        }
                        _ => todo!("{:?}", MetadataCodes::from_u64(record.code)),
                    },
                    _ => todo!(),
                },
                None => return Ok(()),
            }

            content = self.bitstream.next();
        }
    }

    pub fn parse_operand_bundle_tags(&mut self, result: &mut AIRModule) -> Result<()> {
        let mut content = self.bitstream.next();

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => {
                        return Ok(());
                    }
                    StreamEntry::Record(record) => {
                        if record.code != 1 {
                            return Err(anyhow!("Invalid operand bundle tag."));
                        }

                        result
                            .operand_bundle_tags
                            .push(Self::parse_string(record.fields));
                    }
                    _ => todo!(),
                },
                None => return Ok(()),
            }

            content = self.bitstream.next();
        }
    }

    pub fn parse_sync_scope_names(&mut self, result: &mut AIRModule) -> Result<()> {
        let mut content = self.bitstream.next();

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => {
                        return Ok(());
                    }
                    StreamEntry::Record(record) => {
                        if record.code != 1 {
                            return Err(anyhow!("Invalid sync scoped name."));
                        }

                        result
                            .sync_scope_names
                            .push(Self::parse_string(record.fields));
                    }
                    _ => todo!(),
                },
                None => return Ok(()),
            }

            content = self.bitstream.next();
        }
    }

    pub fn parse_function_body(&mut self, result: &mut AIRModule, block: Block) -> Result<()> {
        let mut content = self.bitstream.next();
        let mut id = 0;

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => {
                        return Ok(());
                    }
                    StreamEntry::Record(record) => match FunctionCodes::from_u64(record.code) {
                        FunctionCodes::DECLAREBLOCKS => {
                            if record.fields[0] == 0 {
                                return Err(anyhow!("Invalid Declare Block value."));
                            }

                            id = record.fields[0] + 1;
                        }
                        FunctionCodes::INST_CAST => {
                            let first = result.constants.get(&record.fields[0]);
                            let second = result.types[record.fields[1] as usize].clone();
                            todo!("{:?} | {:?} -> {:?}", record.fields, first, second);
                        }
                        _ => todo!("{:?}", FunctionCodes::from_u64(record.code)),
                    },
                    StreamEntry::SubBlock(sub_block) => {
                        match BlockID::from_u64(sub_block.block_id) {
                            BlockID::CONSTANTS => self.parse_constants(result)?,
                            BlockID::METADATA => self.parse_metadata_block(result)?,
                            _ => todo!("{:?}", BlockID::from_u64(sub_block.block_id)),
                        }
                    }
                    _ => todo!(),
                },
                None => return Ok(()),
            }

            content = self.bitstream.next();
        }
    }

    pub fn parse_module_sub_block(
        &mut self,
        sub_block: Block,
        result: &mut AIRModule,
    ) -> Result<()> {
        match BlockID::from_u64(sub_block.block_id) {
            BlockID::TYPE_NEW => result.types = self.parse_type_entries()?,
            BlockID::PARAMATTR_GROUP => result.attributes = self.parse_attribute_group()?,
            BlockID::PARAMATTR => result.entry_table = self.parse_entry_table(result)?,
            BlockID::CONSTANTS => self.parse_constants(result)?,
            BlockID::METADATA_KIND => self.parse_metadata_kind_block(result)?,
            BlockID::METADATA => self.parse_metadata_block(result)?,
            BlockID::OPERAND_BUNDLE_TAGS => self.parse_operand_bundle_tags(result)?,
            BlockID::SYNC_SCOPE_NAMES => self.parse_sync_scope_names(result)?,
            BlockID::FUNCTION => self.parse_function_body(result, sub_block)?,
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
                        self.parse_module_record(record, &mut result)?;
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
