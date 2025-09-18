use std::{collections::HashMap, default, hash::Hash, thread::current};

use anyhow::{Result, anyhow};

use crate::{
    air_parser::{
        AirConstant, AirConstantId, AirConstantValue, AirFile, AirFunctionSignature,
        AirFunctionSignatureId, AirFunctionType, AirGlobalVariableId, AirItem, AirMetadataConstant,
        AirMetadataNamedNode, AirModule, AirType, AirTypeId, AirValue,
    },
    spirv_builder::SpirVBuilder,
    spirv_parser::{
        SpirVAddressingModel, SpirVBuiltIn, SpirVCapability, SpirVConstant, SpirVConstantComposite,
        SpirVConstantValue, SpirVDecorate, SpirVDecorateType, SpirVEntryPoint, SpirVExecutionModel,
        SpirVMemoryModel, SpirVOp, SpirVStorageClass, SpirVType, SpirVVariableId,
    },
};

pub struct AirToSpirV {
    input: AirFile,
    output: Vec<SpirVOp>,
}

impl AirToSpirV {
    pub fn new(input: AirFile) -> Self {
        Self {
            input,
            output: vec![],
        }
    }

    pub fn parse_air_type(
        builder: &mut SpirVBuilder,
        module: &AirModule,
        value: &AirType,
    ) -> SpirVVariableId {
        match value {
            AirType::Void => builder.new_type(SpirVType::Void),
            AirType::Integer(width) => builder.new_type(SpirVType::Int(*width as u32, false)),
            AirType::Float => builder.new_type(SpirVType::Float(32)),
            AirType::Function(function_ty) => {
                let return_ty = Self::parse_air_type(
                    builder,
                    module,
                    &module.types[function_ty.return_type.0 as usize],
                );
                let args = function_ty
                    .params
                    .iter()
                    .map(|ty| Self::parse_air_type(builder, module, &module.types[ty.0 as usize]))
                    .collect::<Vec<_>>();

                builder.new_type(SpirVType::Function(return_ty, args))
            }
            AirType::Struct(struct_ty) => {
                let elements = struct_ty
                    .elements
                    .iter()
                    .map(|ty| {
                        (
                            String::new(),
                            Self::parse_air_type(builder, module, &module.types[ty.0 as usize]),
                        )
                    })
                    .collect::<Vec<_>>();

                builder.new_struct_type(&struct_ty.name, struct_ty.name.is_empty(), elements)
            }
            AirType::Array(array_ty) => {
                let element_ty = Self::parse_air_type(
                    builder,
                    module,
                    &module.types[array_ty.element_type.0 as usize],
                );
                builder.new_type(SpirVType::Array(element_ty, array_ty.size as u32))
            }
            AirType::Vector(vector_ty) => {
                let element_ty = Self::parse_air_type(
                    builder,
                    module,
                    &module.types[vector_ty.element_type.0 as usize],
                );
                builder.new_type(SpirVType::Vector(element_ty, vector_ty.size as u32))
            }
            _ => todo!("{:?}", value),
        }
    }

    pub fn parse_air_constant(
        builder: &mut SpirVBuilder,
        module: &AirModule,
        type_id: SpirVVariableId,
        constant: Option<AirConstant>,
        constant_value: Option<AirConstantValue>,
    ) -> SpirVVariableId {
        let const_val = match constant {
            Some(c) => c.value,
            None => match constant_value {
                Some(cv) => cv,
                None => panic!("No Constant or Constant Value Found."),
            },
        };

        match const_val {
            AirConstantValue::Integer(value) => builder.new_constant(SpirVConstant {
                type_id,
                value: SpirVConstantValue::UnsignedInteger(value),
            }),
            AirConstantValue::Float32(value) => builder.new_constant(SpirVConstant {
                type_id,
                value: SpirVConstantValue::Float32(value),
            }),
            AirConstantValue::Undefined | AirConstantValue::Null | AirConstantValue::Poison => {
                builder.new_constant(SpirVConstant {
                    type_id,
                    value: SpirVConstantValue::Null,
                })
            }
            AirConstantValue::Array(elements) => {
                let values = elements
                    .iter()
                    .map(|value| {
                        Self::parse_air_constant(
                            builder,
                            module,
                            type_id,
                            None,
                            Some(value.clone()),
                        )
                    })
                    .collect();

                builder.new_constant_composite(SpirVConstantComposite { type_id, values })
            }
            AirConstantValue::Aggregate(elements) => {
                let values = elements
                    .iter()
                    .map(|value| match module.constants.get(value) {
                        Some(value) => Self::parse_air_constant(
                            builder,
                            module,
                            type_id,
                            Some(value.clone()),
                            None,
                        ),
                        None => Self::parse_air_constant(
                            builder,
                            module,
                            type_id,
                            None,
                            Some(AirConstantValue::Poison),
                        ),
                    })
                    .collect();

                builder.new_constant_composite(SpirVConstantComposite { type_id, values })
            }
            _ => todo!("{:?}", const_val),
        }
    }

    pub fn codegen_functions(
        module: &AirModule,
        builder: &mut SpirVBuilder,
        entry_points: &HashMap<AirFunctionSignatureId, SpirVVariableId>,
    ) -> Result<()> {
        for (air_id, spirv_type_id) in entry_points {
            let air_function_signature = module.get_function_signature(*air_id).unwrap();
            let mut function_body = None;
            for i in &module.function_bodies {
                if i.signature == *air_id {
                    function_body = Some(i);
                }
            }

            let air_function_name = module.string_table[air_function_signature.name.0 as usize]
                .content
                .clone();

            let mut arguments = vec![];
            let air_function_body = function_body.unwrap();
            match builder
                .module
                .type_table
                .get(spirv_type_id)
                .unwrap()
                .clone()
            {
                SpirVType::Function(output, inputs) => {
                    let new_output_type =
                        builder.new_type(SpirVType::Pointer(SpirVStorageClass::Output, output));

                    arguments.push(builder.new_variable(
                        &(air_function_name.clone() + "_air_output"),
                        new_output_type,
                        SpirVStorageClass::Output,
                        None,
                    ));

                    let mut input_count = 0;
                    for i in inputs {
                        let new_input_type =
                            builder.new_type(SpirVType::Pointer(SpirVStorageClass::Input, i));

                        arguments.push(builder.new_variable(
                            &(air_function_name.clone() + "_air_input_" + &input_count.to_string()),
                            new_input_type,
                            SpirVStorageClass::Input,
                            None,
                        ));

                        input_count += 1;
                    }
                }
                _ => todo!(),
            }

            let spirv_final_function_type = builder.new_type(SpirVType::Void);
            let spirv_final_function_pointer =
                builder.new_type(SpirVType::Function(spirv_final_function_type, vec![]));

            let function = builder.new_function(
                &air_function_name,
                spirv_final_function_pointer,
                spirv_final_function_type,
                vec![],
            );

            builder.new_entry_point(
                &air_function_name,
                function,
                SpirVExecutionModel::Vertex,
                arguments,
            );
        }

        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        let mut module: Option<AirModule> = None;

        for i in &self.input.items {
            match i {
                AirItem::Module(m) => module = Some(m.clone()),
                _ => continue,
            }
        }

        if module.is_none() {
            return Err(anyhow!("Module not found."));
        }

        let module = module.unwrap();
        let mut builder = SpirVBuilder::new();

        builder.set_version(1, 0);
        builder.add_capability(SpirVCapability::Shader);
        builder.add_memory_model(SpirVAddressingModel::Logical, SpirVMemoryModel::Glsl450);

        let mut constants: HashMap<AirConstantId, SpirVVariableId> = HashMap::new();
        for (id, constant) in &module.constants {
            let ty =
                Self::parse_air_type(&mut builder, &module, &module.types[constant.ty.0 as usize]);
            let constant =
                Self::parse_air_constant(&mut builder, &module, ty, Some(constant.clone()), None);
            constants.insert(*id, constant);
        }

        let mut global_variables: HashMap<AirGlobalVariableId, SpirVVariableId> = HashMap::new();
        for (id, global_var) in &module.global_variables {
            let ty = Self::parse_air_type(
                &mut builder,
                &module,
                &module.types[global_var.type_id.0 as usize],
            );
            global_variables.insert(
                *id,
                builder.new_variable(
                    &module.string_table[global_var.name.0 as usize].content,
                    ty,
                    SpirVStorageClass::Private,
                    constants.get(&global_var.initializer).cloned(),
                ),
            );
        }

        let mut entry_points: HashMap<AirFunctionSignatureId, SpirVVariableId> = HashMap::new();
        let mut air_vertex: Option<AirMetadataNamedNode> = None;

        for i in &module.metadata_named_nodes {
            if i.name == "air.vertex" {
                air_vertex = Some(i.clone());
            }
        }

        match air_vertex {
            Some(air_vertex) => {
                for i in air_vertex.operands {
                    let entry = match &module.metadata_constants[&i] {
                        AirMetadataConstant::Node(entry) => entry,
                        _ => panic!("Expected Node, found {:?}", module.metadata_constants[&i]),
                    };

                    let function_signature = match module.metadata_constants.get(&entry[0]).unwrap()
                    {
                        AirMetadataConstant::Value(value) => match value {
                            AirValue::Function(function) => {
                                module.get_function_signature(*function).unwrap()
                            }
                            _ => panic!("Expected Function, found {:?}", value),
                        },
                        _ => {
                            panic!(
                                "Expected Value, found {:?}",
                                module.metadata_constants[&entry[0]]
                            )
                        }
                    };

                    let entry_point_outputs =
                        match module.metadata_constants.get(&entry[1]).unwrap() {
                            AirMetadataConstant::Node(entry_point_outputs) => entry_point_outputs,
                            _ => {
                                panic!(
                                    "Expected Node Group, found {:?}",
                                    module.metadata_constants[&entry[1]]
                                )
                            }
                        };

                    let entry_point_inputs = match module.metadata_constants.get(&entry[2]).unwrap()
                    {
                        AirMetadataConstant::Node(entry_point_inputs) => entry_point_inputs,
                        _ => {
                            panic!(
                                "Expected Node Group, found {:?}",
                                module.metadata_constants[&entry[2]]
                            )
                        }
                    };

                    let mut vertex_info_output =
                        Self::parse_vertex_info(&module, entry_point_outputs.clone());

                    vertex_info_output
                        .merge(Self::parse_vertex_info(&module, entry_point_inputs.clone()));

                    let air_function_type = Self::parse_air_type(
                        &mut builder,
                        &module,
                        &AirType::Function(function_signature.ty.clone()),
                    );

                    let mut location_count = 0;
                    let mut variable_count = 0;
                    match &builder.module.type_table[&air_function_type].clone() {
                        SpirVType::Function(output, inputs) => {
                            let output = vec![*output];
                            let mut arguments = Self::parse_entry_point_variable(
                                &mut builder,
                                &output,
                                &vertex_info_output,
                                &mut location_count,
                                &mut variable_count,
                            );

                            arguments.extend(Self::parse_entry_point_variable(
                                &mut builder,
                                inputs,
                                &vertex_info_output,
                                &mut location_count,
                                &mut variable_count,
                            ));

                            dbg!(arguments);
                        }
                        _ => panic!(
                            "Expected Function, found {:?}",
                            &builder.module.type_table[&air_function_type]
                        ),
                    }

                    entry_points.insert(function_signature.global_id, air_function_type);
                }
            }
            None => {}
        }

        Self::codegen_functions(&module, &mut builder, &entry_points)?;

        todo!("{:#?}", builder.module);

        Ok(())
    }

    pub fn shader_variable_to_spirv_variable(
        builder: &mut SpirVBuilder,
        location: &mut u32,
        current_ty: SpirVVariableId,
        element_info: &ShaderVariable,
    ) -> SpirVVariableId {
        match &element_info.ty {
            ShaderVariableType::Output(output) => match output {
                ShaderOutputType::VertexOutput => {
                    let output_pointer =
                        builder.new_type(SpirVType::Pointer(SpirVStorageClass::Output, current_ty));

                    let output_var = builder.new_variable(
                        &element_info.name.clone(),
                        output_pointer,
                        SpirVStorageClass::Output,
                        None,
                    );

                    let location_ty = SpirVDecorateType::Location(*location);
                    *location += 1;

                    builder.set_decorate(
                        output_var,
                        SpirVDecorate {
                            ty: location_ty,
                            member_decorates: vec![],
                        },
                    );

                    output_var
                }
                ShaderOutputType::Position => {
                    let float = builder.new_type(SpirVType::Float(32));
                    let clip_cull_array = builder.new_type(SpirVType::Array(float, 1));
                    let spirv_position_type = builder.new_struct_type(
                        &element_info.name.clone(),
                        false,
                        vec![
                            ("Position".to_string(), current_ty),
                            ("PointSize".to_string(), float),
                            ("ClipDistance".to_string(), clip_cull_array),
                            ("CullDistance".to_string(), clip_cull_array),
                        ],
                    );

                    builder.set_decorate(
                        spirv_position_type,
                        SpirVDecorate {
                            ty: SpirVDecorateType::Block,
                            member_decorates: vec![
                                SpirVDecorateType::BuiltIn(SpirVBuiltIn::Position),
                                SpirVDecorateType::BuiltIn(SpirVBuiltIn::PointSize),
                                SpirVDecorateType::BuiltIn(SpirVBuiltIn::ClipDistance),
                                SpirVDecorateType::BuiltIn(SpirVBuiltIn::CullDistance),
                            ],
                        },
                    );

                    let pointer = builder.new_type(SpirVType::Pointer(
                        SpirVStorageClass::Output,
                        spirv_position_type,
                    ));

                    builder.new_variable("VertexOutput", pointer, SpirVStorageClass::Output, None)
                }
            },
            ShaderVariableType::Input(input) => match input {
                ShaderInputType::VertexInput => {
                    let input_pointer =
                        builder.new_type(SpirVType::Pointer(SpirVStorageClass::Input, current_ty));

                    let input_var = builder.new_variable(
                        &element_info.name.clone(),
                        input_pointer,
                        SpirVStorageClass::Input,
                        None,
                    );

                    let location_ty = SpirVDecorateType::Location(*location);
                    *location += 1;

                    builder.set_decorate(
                        input_var,
                        SpirVDecorate {
                            ty: location_ty,
                            member_decorates: vec![],
                        },
                    );

                    input_var
                }
                ShaderInputType::VertexID => {
                    let input_pointer =
                        builder.new_type(SpirVType::Pointer(SpirVStorageClass::Input, current_ty));

                    let vertex_id = builder.new_variable(
                        &element_info.name.clone(),
                        input_pointer,
                        SpirVStorageClass::Input,
                        None,
                    );

                    builder.set_decorate(
                        vertex_id,
                        SpirVDecorate {
                            ty: SpirVDecorateType::BuiltIn(SpirVBuiltIn::VertexId),
                            member_decorates: vec![],
                        },
                    );

                    vertex_id
                }
            },
            _ => todo!("{:?}", element_info),
        }
    }

    pub fn parse_entry_point_variable(
        builder: &mut SpirVBuilder,
        inputs: &Vec<SpirVVariableId>,
        info: &VertexFunctionInfo,
        variable_count: &mut usize,
        location: &mut u32,
    ) -> Vec<SpirVVariableId> {
        let mut result: Vec<SpirVVariableId> = vec![];
        for i in inputs {
            let input = builder.module.type_table.get(&i).unwrap().clone();
            match input {
                SpirVType::Struct(elements) => {
                    for i in elements {
                        let element_info = &info.variables[*variable_count];
                        result.push(Self::shader_variable_to_spirv_variable(
                            builder,
                            location,
                            i,
                            element_info,
                        ));
                        *variable_count += 1;
                    }
                }
                _ => {
                    let element_info = &info.variables[*variable_count];
                    result.push(Self::shader_variable_to_spirv_variable(
                        builder,
                        location,
                        *i,
                        element_info,
                    ));
                    *variable_count += 1;
                }
            }
        }

        result
    }

    pub fn parse_metadata_value(
        properties: &Vec<u64>,
        module: &AirModule,
        value_name: &mut String,
        start_at: usize,
    ) {
        let mut count = start_at;

        loop {
            if count >= properties.len() {
                break;
            }

            let variable_string = module.get_metadata_string(properties[count]).unwrap();

            // TODO: Handle user defined locations.
            if !variable_string.starts_with("user") {
                match variable_string.as_str() {
                    "air.arg_type_name" => {
                        // Skip, since we already have the AIR/LLVM Type.
                        // There's no need to do string parsing or anything of the sort.
                        count += 1;
                    }
                    "air.arg_name" => {
                        count += 1;
                        *value_name = module.get_metadata_string(properties[count]).unwrap();
                    }
                    _ => {
                        todo!("{:?}", variable_string)
                    }
                }
            }

            count += 1;
        }
    }

    pub fn parse_vertex_info(
        module: &AirModule,
        vertex_values_info: Vec<u64>,
    ) -> VertexFunctionInfo {
        let mut variables: Vec<ShaderVariable> = vec![];
        let mut location_id = 0;
        for i in vertex_values_info {
            let vertex_properties = match module.metadata_constants.get(&i).unwrap() {
                AirMetadataConstant::Node(vertex_properties) => vertex_properties,
                _ => {
                    panic!(
                        "Expected Node Group, found {:?}",
                        module.metadata_constants[&i]
                    )
                }
            };

            dbg!("{:?}", vertex_properties);

            // TODO: Make a more elegant solution if a string isn't found.
            let mut starts_at_two = false;
            let variable_name = match module.get_metadata_string(vertex_properties[0]) {
                Some(v) => v,
                None => {
                    starts_at_two = true;
                    module.get_metadata_string(vertex_properties[1]).unwrap()
                }
            };

            let mut vertex_variable = ShaderVariable::default();
            let mut variable_location_id = None;
            match variable_name.as_str() {
                "air.vertex_output" => {
                    vertex_variable.ty = ShaderVariableType::Output({
                        variable_location_id = Some(location_id);
                        location_id += 1;
                        ShaderOutputType::VertexOutput
                    })
                }
                "air.position" => {
                    vertex_variable.ty = ShaderVariableType::Output(ShaderOutputType::Position)
                }
                "air.vertex_id" => {
                    vertex_variable.ty = ShaderVariableType::Input(ShaderInputType::VertexID)
                }
                _ => todo!("{:?}", variable_name),
            }

            vertex_variable.location = variable_location_id;

            Self::parse_metadata_value(
                vertex_properties,
                module,
                &mut vertex_variable.name,
                if starts_at_two { 2 } else { 1 },
            );
            variables.push(vertex_variable);
        }

        VertexFunctionInfo { variables }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ShaderVariable {
    pub ty: ShaderVariableType,
    pub name: String,
    pub location: Option<u64>,
}

#[derive(Debug, Default, Clone)]
pub enum ShaderVariableType {
    #[default]
    Unknown,
    Input(ShaderInputType),
    Output(ShaderOutputType),
}

#[derive(Debug, Default, Clone)]
pub enum ShaderOutputType {
    #[default]
    VertexOutput,
    Position,
}

#[derive(Debug, Default, Clone)]
pub enum ShaderInputType {
    #[default]
    VertexInput,
    VertexID,
}

#[derive(Debug, Default, Clone)]
pub struct PositionOutput {
    pub name: String,
    pub type_id: SpirVVariableId,
}

#[derive(Debug, Default, Clone)]
pub struct VertexFunctionInfo {
    pub variables: Vec<ShaderVariable>,
}

impl VertexFunctionInfo {
    pub fn merge(&mut self, with: VertexFunctionInfo) {
        self.variables.extend(with.variables);
    }
}
