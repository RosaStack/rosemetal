use crate::spirv_parser::{
    FunctionControl, SpirVAccessChain, SpirVAddressingModel, SpirVAlloca, SpirVBitCast, SpirVBlock,
    SpirVCapability, SpirVCompositeExtract, SpirVCompositeInsert, SpirVConstant,
    SpirVConstantComposite, SpirVConstantValue, SpirVDecorate, SpirVDecorateType, SpirVEntryPoint,
    SpirVExecutionModel, SpirVFunction, SpirVLoad, SpirVMemoryModel, SpirVModule, SpirVName,
    SpirVOp, SpirVOpCode, SpirVSource, SpirVStorageClass, SpirVStore, SpirVType, SpirVVariableId,
    SpirVVectorShuffle,
};

#[derive(Debug, Default, Clone)]
pub struct SpirVBuilder {
    pub module: SpirVModule,
    current_variable_id: u32,
    block_list: Vec<SpirVBlock>,
}

impl SpirVBuilder {
    pub fn new() -> Self {
        Self {
            module: SpirVModule::default(),
            current_variable_id: 1,
            block_list: vec![],
        }
    }

    pub fn id_check<'a>(i: &'a SpirVOp, id: SpirVVariableId) -> Option<&'a SpirVOp> {
        match i {
            SpirVOp::Type(nid, ..)
            | SpirVOp::Constant(nid, ..)
            | SpirVOp::ConstantComposite(nid, ..)
            | SpirVOp::Alloca(nid, ..)
            | SpirVOp::Block(nid, ..)
            | SpirVOp::Load(nid, ..)
            | SpirVOp::AccessChain(nid, ..)
            | SpirVOp::CompositeExtract(nid, ..)
            | SpirVOp::CompositeInsert(nid, ..)
            | SpirVOp::CompositeConstruct(nid, ..)
            | SpirVOp::Function(nid, ..)
            | SpirVOp::BitCast(nid, ..)
            | SpirVOp::VectorShuffle(nid, ..) => {
                if *nid == id {
                    return Some(i);
                }

                None
            }
            _ => None,
        }
    }

    pub fn id_mut_check<'a>(i: &'a mut SpirVOp, id: SpirVVariableId) -> Option<&'a mut SpirVOp> {
        match i {
            SpirVOp::Type(nid, ..)
            | SpirVOp::Constant(nid, ..)
            | SpirVOp::ConstantComposite(nid, ..)
            | SpirVOp::Alloca(nid, ..)
            | SpirVOp::Block(nid, ..)
            | SpirVOp::Load(nid, ..)
            | SpirVOp::AccessChain(nid, ..)
            | SpirVOp::CompositeExtract(nid, ..)
            | SpirVOp::CompositeInsert(nid, ..)
            | SpirVOp::CompositeConstruct(nid, ..)
            | SpirVOp::Function(nid, ..)
            | SpirVOp::BitCast(nid, ..)
            | SpirVOp::VectorShuffle(nid, ..) => {
                if *nid == id {
                    return Some(i);
                }

                None
            }
            _ => None,
        }
    }

    pub fn find_operand_with_id<'a>(&'a self, id: SpirVVariableId) -> &'a SpirVOp {
        for i in &self.block_list {
            for j in &i.instructions {
                match Self::id_check(j, id) {
                    Some(s) => return s,
                    None => {}
                }
            }
        }

        for i in &self.module.operands {
            match Self::id_check(i, id) {
                Some(s) => return s,
                None => {}
            }
        }

        panic!("ID {:?} not found.", id)
    }

    pub fn find_mut_operand_with_id<'a>(&'a mut self, id: SpirVVariableId) -> &'a mut SpirVOp {
        for i in &mut self.block_list {
            for j in &mut i.instructions {
                match Self::id_mut_check(j, id) {
                    Some(s) => return s,
                    None => {}
                }
            }
        }

        for i in &mut self.module.operands {
            match Self::id_mut_check(i, id) {
                Some(s) => return s,
                None => {}
            }
        }

        panic!("ID {:?} not found.", id)
    }

    pub fn find_operand_type_id<'a>(&'a self, id: SpirVVariableId) -> SpirVVariableId {
        let find_id = self.find_operand_with_id(id);

        match find_id {
            SpirVOp::Alloca(_, alloca) => alloca.type_id,
            _ => todo!(),
        }
    }

    pub fn find_pointer_type<'a>(&'a self, id: SpirVVariableId) -> SpirVVariableId {
        match self.module.type_table.get(&id).unwrap() {
            SpirVType::Pointer(_, fid) => *fid,
            _ => id,
        }
    }

    pub fn next_id(&mut self) -> SpirVVariableId {
        self.current_variable_id += 1;
        SpirVVariableId(self.current_variable_id)
    }

    pub fn set_version(&mut self, major: u8, minor: u8) {
        self.module.signature.version = (major, minor);
    }

    pub fn add_capability(&mut self, capability: SpirVCapability) {
        self.module.capabilities.push(capability.clone());
        self.module.operands.push(SpirVOp::Capability(capability));
    }

    pub fn add_memory_model(
        &mut self,
        addressing_model: SpirVAddressingModel,
        memory_model: SpirVMemoryModel,
    ) {
        self.module.addressing_model = Some(addressing_model.clone());
        self.module.memory_model = Some(memory_model.clone());

        self.module
            .operands
            .push(SpirVOp::MemoryModel(addressing_model, memory_model));
    }

    pub fn add_source(&mut self, source: SpirVSource) {
        self.module.operands.push(SpirVOp::Source(source));
    }

    pub fn new_extended_instruction_import(&mut self, import_name: &str) {
        let import_name = import_name.to_string();

        let id = self.new_id();

        self.module
            .operands
            .push(SpirVOp::ExtendedInstructionImport(id, import_name));
    }

    pub fn new_source_extension(&mut self, extension_name: &str) {
        self.module
            .operands
            .push(SpirVOp::SourceExtension(extension_name.to_string()));
    }

    pub fn new_type(&mut self, ty: SpirVType) -> SpirVVariableId {
        let var = SpirVVariableId(self.current_variable_id);

        for (id, table_ty) in &self.module.type_table {
            if ty == *table_ty {
                return *id;
            }
        }

        self.module.type_table.insert(var, ty.clone());
        self.module.operands.push(SpirVOp::Type(var, ty));

        self.current_variable_id += 1;

        var
    }

    pub fn new_entry_point(
        &mut self,
        name: &str,
        function_id: SpirVVariableId,
        execution_model: SpirVExecutionModel,
        arguments: Vec<SpirVVariableId>,
    ) -> SpirVVariableId {
        let var = SpirVVariableId(self.current_variable_id);

        let entry_point = SpirVEntryPoint {
            name: name.to_string(),
            execution_model,
            entry_point_id: function_id,
            arguments,
        };

        self.module
            .entry_point_table
            .insert(var, entry_point.clone());

        self.module.operands.push(SpirVOp::EntryPoint(entry_point));

        self.current_variable_id += 1;

        var
    }

    pub fn new_variable(
        &mut self,
        name: &str,
        type_id: SpirVVariableId,
        storage_class: SpirVStorageClass,
        initializer: Option<SpirVVariableId>,
    ) -> SpirVVariableId {
        let var = SpirVVariableId(self.current_variable_id);
        let alloca = SpirVAlloca {
            type_id,
            storage_class,
            initializer,
        };

        self.module.alloca_table.insert(var, alloca.clone());

        if !name.is_empty() {
            self.module.name_table.insert(
                var,
                SpirVName {
                    name: name.to_string(),
                    member_names: vec![],
                },
            );
        }

        self.module
            .operands
            .push(SpirVOp::Name(var, name.to_string()));
        self.module.operands.push(SpirVOp::Alloca(var, alloca));

        self.current_variable_id += 1;

        var
    }

    pub fn new_struct_type(
        &mut self,
        name: &str,
        _is_packed: bool,
        elements: Vec<(String, SpirVVariableId)>,
    ) -> SpirVVariableId {
        let var = SpirVVariableId(self.current_variable_id);

        let mut elements_ty = vec![];
        let mut names_ty = vec![];

        for i in elements {
            names_ty.push(i.0);
            elements_ty.push(i.1);
        }

        let spirv_name = SpirVName {
            name: name.to_string(),
            member_names: names_ty,
        };

        let final_struct_ty = SpirVType::Struct(elements_ty);

        for (id, table_ty) in &self.module.type_table {
            if table_ty == &final_struct_ty {
                if *self.module.name_table.get(id).unwrap() == spirv_name {
                    return *id;
                }
            }
        }

        self.module.name_table.insert(var, spirv_name.clone());

        self.module
            .operands
            .push(SpirVOp::Name(var, name.to_string()));

        let mut count = 0;
        for i in spirv_name.member_names {
            self.module
                .operands
                .push(SpirVOp::MemberName(var, count, i.clone()));
            count += 1;
        }

        self.module.type_table.insert(var, final_struct_ty);

        self.current_variable_id += 1;

        var
    }

    pub fn new_constant(&mut self, constant: SpirVConstant) -> SpirVVariableId {
        let var = SpirVVariableId(self.current_variable_id);

        for (id, c) in &self.module.constants_table {
            if c == &constant {
                return *id;
            }
        }

        self.module.constants_table.insert(var, constant.clone());

        self.module.operands.push(SpirVOp::Constant(var, constant));

        self.current_variable_id += 1;

        var
    }

    pub fn set_decorate(&mut self, member_id: SpirVVariableId, decorate: SpirVDecorate) {
        self.module
            .decorate_table
            .insert(member_id, decorate.clone());

        self.module
            .operands
            .push(SpirVOp::Decorate(member_id, decorate.ty));

        let mut count = 0;
        for i in decorate.member_decorates {
            self.module
                .operands
                .push(SpirVOp::MemberDecorate(member_id, count, i));
            count += 1;
        }
    }

    pub fn new_constant_composite(&mut self, composite: SpirVConstantComposite) -> SpirVVariableId {
        let var = SpirVVariableId(self.current_variable_id);

        for (id, c) in &self.module.constant_composites_table {
            if c == &composite {
                return *id;
            }
        }

        self.module
            .constant_composites_table
            .insert(var, composite.clone());

        self.module
            .operands
            .push(SpirVOp::ConstantComposite(var, composite));

        self.current_variable_id += 1;

        var
    }

    pub fn new_function(
        &mut self,
        name: &str,
        function_type: SpirVVariableId,
        return_type: SpirVVariableId,
    ) -> SpirVVariableId {
        let var = SpirVVariableId(self.current_variable_id);

        self.module.name_table.insert(
            var,
            SpirVName {
                name: name.to_string(),
                member_names: vec![],
            },
        );

        let function = SpirVFunction {
            function_type_id: function_type,
            return_type_id: return_type,
            function_control: FunctionControl::None,
            instructions: vec![],
        };

        self.module.functions_table.insert(var, function.clone());

        self.module.operands.push(SpirVOp::Function(var, function));

        self.current_variable_id += 1;

        var
    }

    pub fn new_basic_block(&mut self) {
        self.block_list.push(SpirVBlock::default());
    }

    pub fn new_bit_cast(&mut self, cast: SpirVBitCast) -> SpirVVariableId {
        let id = self.new_id();

        let current_block = self.block_list.last_mut().unwrap();

        current_block.instructions.push(SpirVOp::BitCast(id, cast));

        id
    }

    pub fn new_access_chain(&mut self, access_chain: SpirVAccessChain) -> SpirVVariableId {
        let id = self.new_id();

        let current_block = self.block_list.last_mut().unwrap();

        current_block
            .instructions
            .push(SpirVOp::AccessChain(id, access_chain));

        id
    }

    pub fn new_load(&mut self, load: SpirVLoad) -> SpirVVariableId {
        let id = self.new_id();

        let current_block = self.block_list.last_mut().unwrap();

        current_block.instructions.push(SpirVOp::Load(id, load));

        id
    }

    pub fn new_store(&mut self, store: SpirVStore) -> SpirVVariableId {
        let id = self.new_id();

        let current_block = self.block_list.last_mut().unwrap();

        current_block.instructions.push(SpirVOp::Store(store));

        id
    }

    pub fn new_vector_shuffle(&mut self, vector_shuffle: SpirVVectorShuffle) -> SpirVVariableId {
        let id = self.new_id();

        let current_block = self.block_list.last_mut().unwrap();

        current_block
            .instructions
            .push(SpirVOp::VectorShuffle(id, vector_shuffle));

        id
    }

    pub fn new_composite_extract(
        &mut self,
        composite_extract: SpirVCompositeExtract,
    ) -> SpirVVariableId {
        let id = self.new_id();

        let current_block = self.block_list.last_mut().unwrap();

        current_block
            .instructions
            .push(SpirVOp::CompositeExtract(id, composite_extract));

        id
    }

    pub fn new_composite_insert(
        &mut self,
        composite_insert: SpirVCompositeInsert,
    ) -> SpirVVariableId {
        let id = self.new_id();

        let current_block = self.block_list.last_mut().unwrap();

        current_block
            .instructions
            .push(SpirVOp::CompositeInsert(id, composite_insert));

        id
    }

    pub fn new_return(&mut self, value: Option<SpirVVariableId>) -> SpirVVariableId {
        let id = self.new_id();

        let current_block = self.block_list.last_mut().unwrap();

        match value {
            Some(s) => current_block.instructions.push(SpirVOp::ReturnValue(id)),
            None => current_block.instructions.push(SpirVOp::Return),
        }

        id
    }

    pub fn new_id(&mut self) -> SpirVVariableId {
        let id = self.current_variable_id;
        self.current_variable_id += 1;
        SpirVVariableId(id)
    }

    pub fn end_function(&mut self, func: SpirVVariableId) -> SpirVVariableId {
        self.block_list
            .last_mut()
            .unwrap()
            .instructions
            .push(SpirVOp::FunctionEnd);

        let id = self.new_id();
        let block = self.block_list.last().unwrap().clone();

        self.module
            .functions_table
            .get_mut(&func)
            .unwrap()
            .instructions
            .push(SpirVOp::Block(id, block.clone()));

        let func_operand = self.find_mut_operand_with_id(func);
        match func_operand {
            SpirVOp::Function(_, function) => function.instructions.push(SpirVOp::Block(id, block)),
            _ => todo!(),
        }

        self.block_list.clear();

        func
    }

    pub fn assemble(&self) -> Vec<u32> {
        let mut result: Vec<u32> = vec![];

        // Magic Number.
        result.push(0x7230203);

        // SPIR-V Version (1.0).
        result.push(u32::from_le_bytes([0_u8, 0_u8, 1_u8, 0_u8]));

        // Generator, Bound and Schema.
        result.extend(vec![0x000D000B, self.current_variable_id, 0]);

        for i in &self.module.operands {
            result.extend(self.assemble_operand(i));
        }

        result
    }

    pub fn assemble_operand(&self, op: &SpirVOp) -> Vec<u32> {
        match op {
            SpirVOp::Capability(capability) => {
                vec![
                    Self::new_opcode(2, SpirVOpCode::Capability),
                    *capability as u32,
                ]
            }
            SpirVOp::MemoryModel(addressing_model, memory_model) => {
                vec![
                    Self::new_opcode(3, SpirVOpCode::MemoryModel),
                    *addressing_model as u32,
                    *memory_model as u32,
                ]
            }
            SpirVOp::Type(id, ty) => match ty {
                SpirVType::Int(width, signedness) => vec![
                    Self::new_opcode(4, SpirVOpCode::TypeInt),
                    id.0,
                    *width,
                    if *signedness { 1 } else { 0 },
                ],
                SpirVType::Float(width) => {
                    vec![Self::new_opcode(3, SpirVOpCode::TypeFloat), id.0, *width]
                }
                SpirVType::Vector(type_id, size) => {
                    vec![
                        Self::new_opcode(4, SpirVOpCode::TypeVector),
                        id.0,
                        type_id.0,
                        *size,
                    ]
                }
                SpirVType::Array(type_id, size) => {
                    vec![
                        Self::new_opcode(4, SpirVOpCode::TypeArray),
                        id.0,
                        type_id.0,
                        *size,
                    ]
                }
                SpirVType::Function(return_type_id, arguments) => {
                    let mut result = vec![
                        Self::new_opcode(3 + arguments.len() as u32, SpirVOpCode::TypeFunction),
                        id.0,
                        return_type_id.0,
                    ];
                    result.extend(arguments.iter().map(|value| value.0).collect::<Vec<_>>());
                    result
                }
                SpirVType::Pointer(storage_class, type_id) => {
                    vec![
                        Self::new_opcode(4, SpirVOpCode::TypePointer),
                        *storage_class as u32,
                        type_id.0,
                    ]
                }
                _ => todo!("{:?}", ty),
            },
            SpirVOp::Constant(id, constant) => vec![
                Self::new_opcode(4, SpirVOpCode::Constant),
                constant.type_id.0,
                id.0,
                match constant.value {
                    SpirVConstantValue::SignedInteger(int) => {
                        u32::from_le_bytes((int as i32).to_le_bytes())
                    }
                    SpirVConstantValue::UnsignedInteger(int) => int as u32,
                    SpirVConstantValue::Float32(float) => u32::from_le_bytes(float.to_le_bytes()),
                    SpirVConstantValue::Float64(float) => {
                        u32::from_le_bytes((float as f32).to_le_bytes())
                    }
                    SpirVConstantValue::Undefined | SpirVConstantValue::Null => {
                        // Set as '0' for now...
                        // Please don't make this a permanent
                        // solution.
                        0
                    }
                    _ => todo!(),
                },
            ],
            SpirVOp::ConstantComposite(id, composite) => {
                let mut result = vec![
                    Self::new_opcode(
                        3 + composite.values.len() as u32,
                        SpirVOpCode::ConstantComposite,
                    ),
                    composite.type_id.0,
                    id.0,
                ];

                result.extend(
                    composite
                        .values
                        .iter()
                        .map(|value| value.0)
                        .collect::<Vec<_>>(),
                );

                result
            }
            SpirVOp::Name(id, name) => {
                let name = Self::string_to_spirv_name(name);
                let mut result = vec![
                    Self::new_opcode(3 + name.len() as u32 - 1, SpirVOpCode::Name),
                    id.0,
                ];
                result.extend(name);
                result
            }
            SpirVOp::MemberName(id, index, name) => {
                let name = Self::string_to_spirv_name(name);
                let mut result = vec![
                    Self::new_opcode(4 + name.len() as u32 - 1, SpirVOpCode::MemberName),
                    id.0,
                    *index as u32,
                ];
                result.extend(name);
                result
            }
            SpirVOp::Alloca(id, alloca) => {
                let mut result = vec![
                    Self::new_opcode(
                        4 + { if alloca.initializer.is_none() { 0 } else { 1 } },
                        SpirVOpCode::Variable,
                    ),
                    alloca.type_id.0,
                    id.0,
                    alloca.storage_class as u32,
                ];

                match alloca.initializer {
                    Some(init) => result.push(init.0),
                    None => {}
                }

                result
            }
            SpirVOp::Decorate(target_id, decorate_type) => {
                let decorate_result = Self::assemble_decorate_type(decorate_type);

                let mut result = vec![
                    Self::new_opcode(3 + decorate_result.len() as u32 - 1, SpirVOpCode::Decorate),
                    target_id.0,
                ];

                result.extend(decorate_result);

                result
            }
            SpirVOp::MemberDecorate(target_id, member_index, decorate_type) => {
                let decorate_result = Self::assemble_decorate_type(decorate_type);

                let mut result = vec![
                    Self::new_opcode(
                        4 + decorate_result.len() as u32 - 1,
                        SpirVOpCode::MemberDecorate,
                    ),
                    target_id.0,
                    *member_index as u32,
                ];

                result.extend(decorate_result);

                result
            }
            SpirVOp::Function(id, function) => {
                let mut result = vec![
                    Self::new_opcode(4, SpirVOpCode::Function),
                    function.return_type_id.0,
                    id.0,
                    function.function_control as u32,
                    function.function_type_id.0,
                ];

                for i in &function.instructions {
                    result.extend(self.assemble_operand(i));
                }

                result
            }
            SpirVOp::Block(id, block) => {
                let mut result = vec![Self::new_opcode(2, SpirVOpCode::Label), id.0];

                for i in &block.instructions {
                    result.extend(self.assemble_operand(i));
                }

                result
            }
            SpirVOp::BitCast(id, bit_cast) => {
                vec![
                    Self::new_opcode(4, SpirVOpCode::BitCast),
                    bit_cast.to_type.0,
                    id.0,
                    bit_cast.variable.0,
                ]
            }
            SpirVOp::AccessChain(id, access_chain) => {
                let mut result = vec![
                    Self::new_opcode(
                        4 + access_chain.indices.len() as u32,
                        SpirVOpCode::AccessChain,
                    ),
                    access_chain.type_id.0,
                    id.0,
                    access_chain.base_id.0,
                ];

                result.extend(
                    access_chain
                        .indices
                        .iter()
                        .map(|value| value.0)
                        .collect::<Vec<_>>(),
                );

                result
            }
            SpirVOp::Load(id, load) => {
                vec![
                    Self::new_opcode(5, SpirVOpCode::Load),
                    load.type_id.0,
                    id.0,
                    load.pointer_id.0,
                    load.memory_operands as u32,
                ]
            }
            SpirVOp::VectorShuffle(id, vector_shuffle) => {
                let mut result = vec![
                    Self::new_opcode(
                        5 + vector_shuffle.mask.len() as u32,
                        SpirVOpCode::VectorShuffle,
                    ),
                    vector_shuffle.vec_type.0,
                    id.0,
                    vector_shuffle.vec1.0,
                    vector_shuffle.vec2.0,
                ];

                result.extend_from_slice(&vector_shuffle.mask);

                result
            }
            SpirVOp::CompositeInsert(id, composite_insert) => {
                let mut result = vec![
                    Self::new_opcode(
                        5 + composite_insert.indices.len() as u32,
                        SpirVOpCode::CompositeInsert,
                    ),
                    composite_insert.type_id.0,
                    id.0,
                    composite_insert.object_id.0,
                    composite_insert.composite_id.0,
                ];

                result.extend_from_slice(&composite_insert.indices);

                result
            }
            SpirVOp::CompositeExtract(id, composite_extract) => {
                let mut result = vec![
                    Self::new_opcode(
                        4 + composite_extract.indices.len() as u32,
                        SpirVOpCode::CompositeExtract,
                    ),
                    composite_extract.type_id.0,
                    id.0,
                    composite_extract.composite_id.0,
                ];

                result.extend_from_slice(&composite_extract.indices);

                result
            }
            SpirVOp::Store(store) => {
                vec![
                    Self::new_opcode(4, SpirVOpCode::Store),
                    store.pointer_id.0,
                    store.object_id.0,
                    store.memory_operands as u32,
                ]
            }
            SpirVOp::Return => vec![Self::new_opcode(1, SpirVOpCode::Return)],
            SpirVOp::FunctionEnd => vec![Self::new_opcode(1, SpirVOpCode::FunctionEnd)],
            SpirVOp::EntryPoint(entry_point) => {
                let name = Self::string_to_spirv_name(&entry_point.name);
                let mut result = vec![
                    Self::new_opcode(
                        4 + (name.len() as u32 - 1) + entry_point.arguments.len() as u32,
                        SpirVOpCode::EntryPoint,
                    ),
                    entry_point.execution_model as u32,
                    entry_point.entry_point_id.0,
                ];

                result.extend(name);

                result.extend(
                    entry_point
                        .arguments
                        .iter()
                        .map(|arg| arg.0)
                        .collect::<Vec<_>>(),
                );

                result
            }
            SpirVOp::Source(source) => {
                vec![
                    Self::new_opcode(3, SpirVOpCode::Source),
                    source.source_language as u32,
                    source.version,
                ]
            }
            SpirVOp::ExtendedInstructionImport(id, name) => {
                let mut name = Self::string_to_spirv_name(name);

                // The Extended Name has to end with
                // an empty word for some reason.
                // Don't ask me why, ask Khronos Group.
                name.push(0);

                let mut result = vec![
                    Self::new_opcode(3 + name.len() as u32 - 1, SpirVOpCode::ExtInstImport),
                    id.0,
                ];

                result.extend(name);

                result
            }
            SpirVOp::SourceExtension(extension_name) => {
                let extension_name = Self::string_to_spirv_name(extension_name);

                let mut result = vec![Self::new_opcode(
                    2 + extension_name.len() as u32 - 1,
                    SpirVOpCode::SourceExtension,
                )];

                result.extend(extension_name);

                result
            }
            _ => todo!("{:?}", op),
        }
    }

    pub fn new_opcode(word_count: u32, op_code: SpirVOpCode) -> u32 {
        let word_count = word_count as u16;
        let op_code = op_code as u16;

        let mut result = vec![];
        result.extend_from_slice(&op_code.to_le_bytes());
        result.extend_from_slice(&word_count.to_le_bytes());

        u32::from_le_bytes([result[0], result[1], result[2], result[3]])
    }

    pub fn assemble_decorate_type(decorate_type: &SpirVDecorateType) -> Vec<u32> {
        match decorate_type {
            SpirVDecorateType::Block => vec![2],
            SpirVDecorateType::BuiltIn(builtin) => vec![11, *builtin as u32],
            SpirVDecorateType::Location(location) => vec![30, *location],
        }
    }

    pub fn string_to_spirv_name(name: &String) -> Vec<u32> {
        let mut result = vec![];
        let mut count = 0;
        let mut integer_buffer = [0_u8, 0_u8, 0_u8, 0_u8];
        for i in name.as_bytes() {
            if count > 3 {
                result.push(u32::from_le_bytes(integer_buffer));
                integer_buffer = [0_u8, 0_u8, 0_u8, 0_u8];
                count = 0;
            }

            integer_buffer[count] = *i;
            count += 1;
        }

        result.push(u32::from_le_bytes(integer_buffer));

        result
    }

    pub fn assemble_to_bytes(&self) -> Vec<u8> {
        let assemble = self.assemble();

        let mut result = vec![];

        for i in assemble {
            result.extend_from_slice(&i.to_le_bytes());
        }

        result
    }
}
