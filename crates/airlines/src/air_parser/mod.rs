pub mod items;

use std::collections::HashMap;

pub use items::*;

use anyhow::{Result, anyhow};

use crate::llvm_bitcode::{
    AttributeCode, AttributeKindCode, BitCursor, Bitstream, Block, BlockID, CastOpCode,
    ConstantsCode, Fields, FunctionCodes, GEPNoWrapFlags, IdentificationCode, MetadataCodes,
    ModuleCode, Record, Signature, StreamEntry, TypeCode,
};

pub struct Parser {
    pub signature: Option<Signature>,
    pub bitstream: Bitstream,
}

impl Parser {
    pub fn new(inner: Vec<u8>) -> Result<Self> {
        let (signature, bitstream) = Bitstream::from(inner)?;

        Ok(Self {
            signature,
            bitstream,
        })
    }

    pub fn parse_blob(fields: Fields) -> AirBlob {
        let mut result: Vec<u8> = vec![];

        for i in fields {
            result.push(i as u8);
        }

        AirBlob { content: result }
    }

    pub fn parse_blob_vec(&mut self) -> Result<Vec<AirBlob>> {
        let mut content = self.bitstream.next();
        let mut result: Vec<AirBlob> = vec![];

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => {
                        return Ok(result);
                    }
                    StreamEntry::Record(record) => {
                        // TODO: Please add a check so it doesn't always
                        // assume that its a blob.
                        result.push(Self::parse_blob(record.fields));
                    }
                    _ => todo!(),
                },
                None => return Ok(result),
            }

            content = self.bitstream.next();
        }
    }

    pub fn parse_string_table_contents(
        &mut self,
        module: &mut Option<AirModule>,
    ) -> Result<AirStringTable> {
        let mut content = self.bitstream.next();
        let mut strings: Vec<String> = vec![];

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => {
                        return Ok(AirStringTable { strings });
                    }
                    StreamEntry::Record(record) => match module {
                        Some(module) => {
                            let string = Self::parse_string(record.fields);
                            for i in &mut module.string_table {
                                let begin = i.offset;
                                let end = i.offset + i.size;
                                i.content = string[begin as usize..end as usize].to_string();
                            }
                            strings.push(string);
                        }
                        None => return Err(anyhow!("Module not found.")),
                    },
                    _ => todo!(),
                },
                None => return Ok(AirStringTable { strings }),
            }

            content = self.bitstream.next();
        }
    }

    pub fn parse_block(&mut self, b: Block, module: &mut Option<AirModule>) -> Result<AirItem> {
        match BlockID::from_u64(b.block_id) {
            BlockID::IDENTIFICATION => Ok(AirItem::IdentificationBlock(
                self.parse_identification_block()?,
            )),
            BlockID::MODULE => Ok(AirItem::Module(self.parse_module()?)),
            BlockID::SYMTAB => Ok(AirItem::SymTabBlock(AirSymTabBlock {
                blobs: self.parse_blob_vec()?,
            })),
            BlockID::STRTAB => Ok(AirItem::StringTable(
                self.parse_string_table_contents(module)?,
            )),
            _ => todo!("{:?} not implemented yet.", b),
        }
    }

    pub fn parse_module_record(&mut self, record: Record, result: &mut AirModule) -> Result<()> {
        match ModuleCode::from_u64(record.code) {
            ModuleCode::VERSION => {
                result.version = record.fields[0];
                result.use_relative_ids = result.version >= 1;
            }
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

    pub fn parse_type_entries(&mut self) -> Result<Vec<AirType>> {
        let mut content = self.bitstream.next();
        let mut result: Vec<AirType> = vec![];
        let mut last_struct_name = String::new();

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => {
                        return Ok(result);
                    }
                    StreamEntry::Record(record) => match TypeCode::from_u64(record.code) {
                        TypeCode::NUMENTRY => {}
                        TypeCode::FLOAT => result.push(AirType::Float),
                        TypeCode::VECTOR => result.push(AirType::Vector(AirVectorType {
                            size: record.fields[0],
                            element_type: AirTypeId(record.fields[1]),
                        })),
                        TypeCode::ARRAY => result.push(AirType::Array(AirArrayType {
                            size: record.fields[0],
                            element_type: AirTypeId(record.fields[1]),
                        })),
                        TypeCode::STRUCT_NAME => {
                            last_struct_name = Self::parse_string(record.fields);
                        }
                        TypeCode::STRUCT_NAMED | TypeCode::STRUCT_ANON => {
                            let mut elements: Vec<AirTypeId> = vec![];

                            let is_packed = record.fields[0] != 0;

                            for i in 1..record.fields.len() {
                                elements.push(AirTypeId(record.fields[i]));
                            }

                            result.push(AirType::Struct(AirStructType {
                                name: last_struct_name.clone(),
                                is_packed,
                                elements,
                            }));

                            last_struct_name.clear();
                        }
                        TypeCode::INTEGER => result.push(AirType::Integer(record.fields[0])),
                        TypeCode::POINTER => result.push(AirType::Pointer(
                            record.fields[1],
                            AirTypeId(record.fields[0]),
                        )),
                        TypeCode::FUNCTION => {
                            let mut param_types: Vec<AirTypeId> = vec![];

                            for i in 2..record.fields.len() {
                                let i = AirTypeId(record.fields[i]);
                                param_types.push(i);
                            }

                            result.push(AirType::Function(AirFunctionType {
                                vararg: record.fields[0],
                                return_type: AirTypeId(record.fields[1]),
                                param_types,
                                param_values: vec![],
                            }));
                        }
                        TypeCode::METADATA => result.push(AirType::Metadata),
                        TypeCode::VOID => result.push(AirType::Void),
                        _ => todo!("{:?}", TypeCode::from_u64(record.code)),
                    },
                    _ => todo!(),
                },
                None => return Ok(result),
            }

            content = self.bitstream.next();
        }
    }

    pub fn parse_attribute(&mut self, record: Record) -> Result<AirAttribute> {
        if record.code == 0 || record.code > 3 {
            return Err(anyhow!("Invalid code for attribute parsing."));
        }

        let mut result = AirAttribute::default();
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
                                .push(AirAttrProperties::WellKnown(property));
                        }
                        3 => {
                            count += 1;
                            let (string, idx) =
                                Self::parse_null_terminated_string(&record.fields, count as u64);

                            count = idx as usize;

                            result
                                .properties
                                .push(AirAttrProperties::StringAttribute(string));
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
                                .push(AirAttrProperties::WithStringValue(first, second));
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

    pub fn parse_attribute_group(&mut self) -> Result<HashMap<u64, AirAttribute>> {
        let mut content = self.bitstream.next();
        let mut result: HashMap<u64, AirAttribute> = HashMap::new();

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

    pub fn parse_entry(&mut self, record: Record, module: &mut AirModule) -> Result<AirAttrEntry> {
        match AttributeCode::from_u64(record.code) {
            AttributeCode::ENTRY => {
                let mut result: Vec<AirAttribute> = vec![];

                for i in record.fields {
                    result.push(module.attributes[&i].clone());
                }

                return Ok(AirAttrEntry { groups: result });
            }
            _ => todo!(),
        }
    }

    pub fn parse_entry_table(
        &mut self,
        module: &mut AirModule,
    ) -> Result<HashMap<u64, AirAttrEntry>> {
        let mut content = self.bitstream.next();
        let mut result: HashMap<u64, AirAttrEntry> = HashMap::new();

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

    pub fn parse_constant_value_with_type(
        result: &mut AirModule,
        ty: AirTypeId,
        value: u64,
    ) -> AirConstantValue {
        match result.types[ty.0 as usize] {
            AirType::Float => {
                let result = f32::from_le_bytes((value as u32).to_le_bytes());
                return AirConstantValue::Float32(result);
            }
            AirType::Integer(_) => {
                return AirConstantValue::Integer(value);
            }
            AirType::Array(_) => {
                return result.constants[&AirConstantId(value)].value.clone();
            }
            AirType::Pointer(_, _) => {
                return AirConstantValue::Pointer(value);
            }
            _ => todo!("{:?}", ty),
        }
    }

    pub fn parse_constant_data(
        &mut self,
        result: &mut AirModule,
        array_ty: AirTypeId,
        fields: Fields,
    ) -> Result<AirConstantValue> {
        let element_type = match result.types[array_ty.0 as usize].clone() {
            AirType::Vector(v) => v.element_type,
            AirType::Array(a) => a.element_type,
            _ => todo!(),
        };

        let mut contents: Vec<AirConstantValue> = vec![];
        for i in fields {
            contents.push(Self::parse_constant_value_with_type(
                result,
                element_type,
                i,
            ));
        }

        Ok(AirConstantValue::Array(contents))
    }

    pub fn decode_sign_rotated_value(v: u64) -> u64 {
        if v & 1 == 0 {
            return v >> 1;
        }

        if v != 1 {
            unsafe {
                return 0_u64.unchecked_sub(v >> 1);
            }
        }

        return 1_u64 << 63;
    }

    pub fn parse_constants(&mut self, module: &mut AirModule) -> Result<()> {
        let mut content = self.bitstream.next();
        let mut current_type = AirTypeId(0);

        let mut skip_add_one_in_settype = false;
        loop {
            match content {
                Some(ucontent) => match ucontent? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => break,
                    StreamEntry::Record(record) => match ConstantsCode::from_u64(record.code) {
                        ConstantsCode::SETTYPE => {
                            current_type = AirTypeId(record.fields[0]);

                            if skip_add_one_in_settype {
                                content = self.bitstream.next();
                                skip_add_one_in_settype = false;
                                continue;
                            }
                        }
                        ConstantsCode::INTEGER => {
                            let _ = module.constants.insert(
                                AirConstantId(module.max_constants_id),
                                AirConstant {
                                    ty: current_type.clone(),
                                    value: AirConstantValue::Integer(
                                        Self::decode_sign_rotated_value(record.fields[0]),
                                    ),
                                },
                            );

                            module.assign_value_to_value_list(
                                module.value_list.len(),
                                AirValue::Constant(AirConstantId(module.max_constants_id)),
                            );
                        }
                        ConstantsCode::NULL => {
                            let _ = module.constants.insert(
                                AirConstantId(module.max_constants_id),
                                AirConstant {
                                    ty: current_type.clone(),
                                    value: AirConstantValue::Null,
                                },
                            );

                            module.assign_value_to_value_list(
                                module.value_list.len(),
                                AirValue::Constant(AirConstantId(module.max_constants_id)),
                            );
                        }
                        ConstantsCode::UNDEF => {
                            let _ = module.constants.insert(
                                AirConstantId(module.max_constants_id),
                                AirConstant {
                                    ty: current_type.clone(),
                                    value: AirConstantValue::Undefined,
                                },
                            );

                            module.assign_value_to_value_list(
                                module.value_list.len(),
                                AirValue::Constant(AirConstantId(module.max_constants_id)),
                            );
                        }
                        ConstantsCode::AGGREGATE => {
                            let mut contents = vec![];
                            for i in record.fields {
                                contents.push(AirValueId(i));
                            }

                            let _ = module.constants.insert(
                                AirConstantId(module.max_constants_id),
                                AirConstant {
                                    ty: current_type.clone(),
                                    value: AirConstantValue::Aggregate(contents),
                                },
                            );
                            skip_add_one_in_settype = true;

                            module.assign_value_to_value_list(
                                module.value_list.len(),
                                AirValue::Constant(AirConstantId(module.max_constants_id)),
                            );
                        }
                        ConstantsCode::DATA => {
                            let data = AirConstant {
                                ty: current_type.clone(),
                                value: self.parse_constant_data(
                                    module,
                                    current_type,
                                    record.fields,
                                )?,
                            };

                            let _ = module
                                .constants
                                .insert(AirConstantId(module.max_constants_id), data);

                            skip_add_one_in_settype = true;

                            module.assign_value_to_value_list(
                                module.value_list.len(),
                                AirValue::Constant(AirConstantId(module.max_constants_id)),
                            );
                        }
                        ConstantsCode::POISON => {
                            let _ = module.constants.insert(
                                AirConstantId(module.max_constants_id),
                                AirConstant {
                                    ty: current_type.clone(),
                                    value: AirConstantValue::Poison,
                                },
                            );

                            module.assign_value_to_value_list(
                                module.value_list.len(),
                                AirValue::Constant(AirConstantId(module.max_constants_id)),
                            );
                        }
                        ConstantsCode::FLOAT => {
                            let _ = module.constants.insert(
                                AirConstantId(module.max_constants_id),
                                AirConstant {
                                    ty: current_type.clone(),
                                    value: AirConstantValue::Float32(f32::from_le_bytes(
                                        (record.fields[0] as u32).to_le_bytes(),
                                    )),
                                },
                            );

                            module.assign_value_to_value_list(
                                module.value_list.len(),
                                AirValue::Constant(AirConstantId(module.max_constants_id)),
                            );
                        }
                        _ => todo!("{:#?}", ConstantsCode::from_u64(record.code)),
                    },

                    _ => todo!(),
                },
                None => break,
            }

            module.max_constants_id += 1;

            content = self.bitstream.next();
        }

        Ok(())
    }

    pub fn parse_metadata_kind_block(&mut self, result: &mut AirModule) -> Result<()> {
        let mut content = self.bitstream.next();

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => return Ok(()),
                    StreamEntry::Record(record) => match MetadataCodes::from_u64(record.code) {
                        MetadataCodes::KIND => {
                            let _ = result.metadata_kind_table.insert(
                                record.fields[0],
                                AirMetadataKind {
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

    pub fn parse_value_symtab(&mut self, _result: &mut AirModule) -> Result<()> {
        let mut content = self.bitstream.next();

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => return Ok(()),
                    StreamEntry::Record(_record) => {
                        // TODO: Parse Value SymTab.
                    }
                    _ => todo!(),
                },
                None => return Ok(()),
            }

            content = self.bitstream.next();
        }
    }

    pub fn parse_metadata_attachment(&mut self, _result: &mut AirModule) -> Result<()> {
        let mut content = self.bitstream.next();

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => return Ok(()),
                    StreamEntry::Record(record) => {
                        if !matches!(
                            MetadataCodes::from_u64(record.code),
                            MetadataCodes::ATTACHMENT
                        ) {
                            return Err(anyhow!("Only accepts Attachments, for now..."));
                        }

                        // TODO: Parse Metadata Attachments.
                    }
                    _ => todo!(),
                },
                None => return Ok(()),
            }

            content = self.bitstream.next();
        }
    }

    pub fn parse_metadata_block(&mut self, result: &mut AirModule) -> Result<()> {
        let mut content = self.bitstream.next();
        let mut current_name = String::new();
        let mut next_metadata_no = result.metadata_constants.len() as u64;
        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => return Ok(()),
                    StreamEntry::Record(record) => match MetadataCodes::from_u64(record.code) {
                        MetadataCodes::STRINGS => {
                            let strings = Self::parse_metadata_strings(record.fields)?;
                            for i in strings {
                                result.metadata_constants.insert(
                                    next_metadata_no,
                                    AirMetadataConstant::String(i.clone()),
                                );
                                next_metadata_no += 1;
                            }
                        }
                        MetadataCodes::INDEX_OFFSET => {
                            result
                                .undiscovered_data
                                .push(UndiscoveredData::INDEX_OFFSET(record.fields[0]));
                        }
                        MetadataCodes::VALUE => {
                            let constant = AirMetadataConstant::Value(
                                result.value_list[record.fields[1] as usize].clone(),
                            );
                            let _ = result.metadata_constants.insert(next_metadata_no, constant);
                            next_metadata_no += 1;
                        }
                        MetadataCodes::NODE => {
                            let _ = result.metadata_constants.insert(
                                next_metadata_no,
                                AirMetadataConstant::Node(
                                    record.fields.iter().map(|x| x - 1).collect(),
                                ),
                            );
                            next_metadata_no += 1;
                        }
                        MetadataCodes::NAMED_NODE => {
                            result.metadata_named_nodes.push(AirMetadataNamedNode {
                                name: current_name.clone(),
                                operands: record.fields,
                            })
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

    pub fn parse_operand_bundle_tags(&mut self, result: &mut AirModule) -> Result<()> {
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

    pub fn parse_sync_scope_names(&mut self, result: &mut AirModule) -> Result<()> {
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

    pub fn get_value(
        &mut self,
        result: &mut AirModule,
        field: u64,
        next_value_no: usize,
    ) -> AirValueId {
        match result.use_relative_ids {
            true => AirValueId(next_value_no as u64 - field),
            false => AirValueId(field),
        }
    }

    pub fn parse_function_body(&mut self, result: &mut AirModule, _block: Block) -> Result<()> {
        let mut content = self.bitstream.next();

        let id = result.current_function_local_id as usize;
        let function_signature = &mut result.function_signatures[id];
        let mut contents: Vec<AirValueId> = vec![];

        let mut count = 0;
        for i in &function_signature.ty.param_types {
            result.value_list.push(AirValue::Argument(AirLocal {
                id: count,
                type_id: *i,
                value: None,
            }));
            function_signature
                .ty
                .param_values
                .push(AirValueId(result.value_list.len() as u64 - 1));
            count += 1;
        }

        let mut next_value_no = result.value_list.len();
        let mut function_body_id = 0;

        loop {
            match content {
                Some(content) => match content? {
                    StreamEntry::EndBlock | StreamEntry::EndOfStream => {
                        break;
                    }
                    StreamEntry::Record(record) => match FunctionCodes::from_u64(record.code) {
                        FunctionCodes::DECLAREBLOCKS => {
                            if record.fields[0] == 0 {
                                return Err(anyhow!("Invalid Declare Block value."));
                            }

                            function_body_id =
                                result.function_bodies.len() + record.fields[0] as usize;
                            result
                                .function_bodies
                                .resize(function_body_id, AirFunctionBody::default());
                        }
                        FunctionCodes::INST_CAST => {
                            let value = self.get_value(result, record.fields[0], next_value_no);
                            let cast_to_type = result.types[record.fields[1] as usize].clone();
                            let cast_code = CastOpCode::from_u64(record.fields[2]);

                            let cast = AirValue::Cast(AirCast {
                                value,
                                cast_to_type,
                                cast_code,
                            });

                            result.value_list.push(cast);
                            contents.push(AirValueId(result.value_list.len() as u64 - 1));

                            next_value_no += 1;
                        }
                        FunctionCodes::INST_GEP => {
                            let no_wrap_flags = GEPNoWrapFlags::from_u64(record.fields[0]);
                            let ty = result.types[record.fields[1] as usize].clone();

                            let base_ptr_value =
                                self.get_value(result, record.fields[2], next_value_no);

                            let mut indices: Vec<AirValueId> = vec![];
                            for i in 3..record.fields.len() {
                                indices.push(self.get_value(
                                    result,
                                    record.fields[i],
                                    next_value_no,
                                ));
                            }

                            let gep = AirValue::GetElementPtr(AirGetElementPtr {
                                no_wrap_flags,
                                ty,
                                base_ptr_value,
                                indices,
                            });

                            result.value_list.push(gep);
                            contents.push(AirValueId(result.value_list.len() as u64 - 1));

                            next_value_no += 1;
                        }
                        FunctionCodes::INST_LOAD => {
                            let op = self.get_value(result, record.fields[0], next_value_no);
                            let ty = result.types[record.fields[1] as usize].clone();

                            let alignment = match record.fields[2].checked_sub(1) {
                                Some(result) => 2_u64.pow(result as u32),
                                None => 0,
                            };

                            let vol = record.fields[3];

                            result.value_list.push(AirValue::Load(AirLoad {
                                op,
                                ty,
                                alignment,
                                vol,
                            }));
                            contents.push(AirValueId(result.value_list.len() as u64 - 1));

                            next_value_no += 1;
                        }
                        FunctionCodes::INST_SHUFFLEVEC => {
                            let vec1 = self.get_value(result, record.fields[0], next_value_no);
                            let vec2 = self.get_value(result, record.fields[1], next_value_no);
                            let mask = self.get_value(result, record.fields[2], next_value_no);

                            result.value_list.push(AirValue::ShuffleVec(AirShuffleVec {
                                vec1,
                                vec2,
                                mask,
                            }));
                            contents.push(AirValueId(result.value_list.len() as u64 - 1));

                            next_value_no += 1;
                        }
                        FunctionCodes::INST_INSERTVAL => {
                            let value1 = self.get_value(result, record.fields[0], next_value_no);
                            let value2 = self.get_value(result, record.fields[1], next_value_no);
                            let insert_value_idx = record.fields[2];

                            result.value_list.push(AirValue::InsertVal(AirInsertVal {
                                value1,
                                value2,
                                insert_value_idx,
                            }));
                            contents.push(AirValueId(result.value_list.len() as u64 - 1));

                            next_value_no += 1;
                        }
                        FunctionCodes::INST_RET => {
                            let value = self.get_value(result, record.fields[0], next_value_no);

                            result
                                .value_list
                                .push(AirValue::Return(AirReturn { value }));
                            contents.push(AirValueId(result.value_list.len() as u64 - 1));

                            next_value_no += 1;
                        }
                        _ => todo!("{:?}", FunctionCodes::from_u64(record.code)),
                    },
                    StreamEntry::SubBlock(sub_block) => {
                        match BlockID::from_u64(sub_block.block_id) {
                            BlockID::CONSTANTS => {
                                self.parse_constants(result)?;
                                next_value_no = result.value_list.len();
                            }
                            BlockID::METADATA => self.parse_metadata_block(result)?,
                            BlockID::METADATA_ATTACHMENT => {
                                self.parse_metadata_attachment(result)?;
                            }
                            _ => todo!("{:?}", BlockID::from_u64(sub_block.block_id)),
                        }
                    }
                },
                None => break,
            }
            content = self.bitstream.next();
        }

        result.current_function_local_id += 1;

        result.function_bodies[function_body_id - 1] = AirFunctionBody {
            signature: result.function_signatures[function_body_id - 1].global_id,
            contents,
        };

        Ok(())
    }

    pub fn parse_module_sub_block(
        &mut self,
        sub_block: Block,
        result: &mut AirModule,
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
            BlockID::VALUE_SYMTAB => self.parse_value_symtab(result)?,
            _ => todo!("{:?}", BlockID::from_u64(sub_block.block_id)),
        }

        Ok(())
    }

    pub fn parse_module(&mut self) -> Result<AirModule> {
        let mut content = self.bitstream.next();
        let mut result = AirModule::default();

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

    pub fn parse_identification_block(&mut self) -> Result<AirIdentificationBlock> {
        let mut content = self.bitstream.next();
        let mut result = AirIdentificationBlock::default();

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

    pub fn next(&mut self, current_module: &mut Option<AirModule>) -> Result<Option<AirItem>> {
        match self.bitstream.next() {
            Some(entry) => match entry? {
                StreamEntry::SubBlock(b) => {
                    let block = self.parse_block(b, current_module)?;
                    Ok(Some(block))
                }
                _ => todo!(),
            },
            None => Ok(None),
        }
    }

    pub fn start(&mut self) -> Result<AirFile> {
        let mut items: Vec<AirItem> = vec![];
        let mut current_module: Option<AirModule> = None;
        let mut content = self.next(&mut current_module)?;

        loop {
            match &content {
                Some(content) => match &content {
                    AirItem::Module(module) => {
                        current_module = Some(module.clone());
                    }
                    _ => {
                        items.push(content.clone());
                    }
                },
                None => break,
            }

            content = self.next(&mut current_module)?;
        }

        items.push(AirItem::Module(current_module.unwrap()));

        Ok(AirFile { items })
    }
}
