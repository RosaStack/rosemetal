use crate::spirv_parser::{
    FunctionControl, SpirVAccessChain, SpirVAddressingModel, SpirVAlloca, SpirVBitCast, SpirVBlock,
    SpirVCapability, SpirVCompositeInsert, SpirVConstant, SpirVConstantComposite, SpirVDecorate,
    SpirVEntryPoint, SpirVExecutionModel, SpirVFunction, SpirVLoad, SpirVMemoryModel, SpirVModule,
    SpirVName, SpirVOp, SpirVStorageClass, SpirVType, SpirVVariableId, SpirVVectorShuffle,
};

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
        contents: Vec<SpirVOp>,
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
            instructions: contents,
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

    pub fn new_vector_shuffle(&mut self, vector_shuffle: SpirVVectorShuffle) -> SpirVVariableId {
        let id = self.new_id();

        let current_block = self.block_list.last_mut().unwrap();

        current_block
            .instructions
            .push(SpirVOp::VectorShuffle(id, vector_shuffle));

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
}
