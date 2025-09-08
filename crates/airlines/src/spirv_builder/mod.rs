use crate::spirv_parser::{
    SpirVAddressingModel, SpirVAlloca, SpirVCapability, SpirVConstant, SpirVConstantComposite,
    SpirVMemoryModel, SpirVModule, SpirVName, SpirVOp, SpirVStorageClass, SpirVType,
    SpirVVariableId,
};

pub struct SpirVBuilder {
    pub module: SpirVModule,
    current_variable_id: u32,
}

impl SpirVBuilder {
    pub fn new() -> Self {
        Self {
            module: SpirVModule::default(),
            current_variable_id: 1,
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

        self.module
            .type_table
            .insert(var, SpirVType::Struct(elements_ty));

        self.current_variable_id += 1;

        var
    }

    pub fn new_constant(&mut self, constant: SpirVConstant) -> SpirVVariableId {
        let var = SpirVVariableId(self.current_variable_id);

        self.module.constants_table.insert(var, constant.clone());

        self.module.operands.push(SpirVOp::Constant(var, constant));

        self.current_variable_id += 1;

        var
    }

    pub fn new_constant_composite(&mut self, composite: SpirVConstantComposite) -> SpirVVariableId {
        let var = SpirVVariableId(self.current_variable_id);

        self.module
            .constant_composites_table
            .insert(var, composite.clone());

        self.module
            .operands
            .push(SpirVOp::ConstantComposite(var, composite));

        self.current_variable_id += 1;

        var
    }
}
