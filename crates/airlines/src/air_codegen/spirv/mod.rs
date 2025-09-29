use std::{collections::HashMap, default, hash::Hash, thread::current};

use anyhow::{Result, anyhow};

use crate::{
    air_codegen::spirv,
    air_parser::{
        AirConstant, AirConstantId, AirConstantValue, AirFile, AirFunctionSignature,
        AirFunctionSignatureId, AirFunctionType, AirGlobalVariableId, AirItem, AirMetadataConstant,
        AirMetadataNamedNode, AirModule, AirReturn, AirType, AirTypeId, AirValue, AirValueId,
        AirVectorType,
    },
    spirv_builder::SpirVBuilder,
    spirv_parser::{
        SpirVAccessChain, SpirVAddressingModel, SpirVBitCast, SpirVBuiltIn, SpirVCapability,
        SpirVCompositeInsert, SpirVConstant, SpirVConstantComposite, SpirVConstantValue,
        SpirVDecorate, SpirVDecorateType, SpirVEntryPoint, SpirVExecutionModel, SpirVLoad,
        SpirVMemoryModel, SpirVMemoryOperands, SpirVOp, SpirVStorageClass, SpirVType,
        SpirVVariableId, SpirVVectorShuffle,
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
                    .param_types
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
                    .map(|value| match module.value_list.get(value.0 as usize) {
                        Some(value) => Self::parse_air_constant(
                            builder,
                            module,
                            type_id,
                            match value {
                                AirValue::Constant(constant) => {
                                    Some(module.constants.get(&constant).unwrap().clone())
                                }
                                _ => None,
                            },
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
        dbg!(&module.constants.len());
        for (id, constant) in &module.constants {
            let ty =
                Self::parse_air_type(&mut builder, &module, &module.types[constant.ty.0 as usize]);
            let constant =
                Self::parse_air_constant(&mut builder, &module, ty, Some(constant.clone()), None);
            constants.insert(*id, constant);
        }
        dbg!(&constants.len());

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
                    let mut spirv_inputs = vec![];
                    let mut spirv_outputs = vec![];
                    let mut air_arguments = vec![];
                    match &builder.module.type_table[&air_function_type].clone() {
                        SpirVType::Function(output, inputs) => {
                            let output = vec![*output];
                            spirv_outputs.extend(Self::parse_entry_point_variable(
                                &mut builder,
                                &output,
                                &vertex_info_output,
                                &mut location_count,
                                &mut variable_count,
                            ));

                            spirv_inputs.extend(Self::parse_entry_point_variable(
                                &mut builder,
                                inputs,
                                &vertex_info_output,
                                &mut location_count,
                                &mut variable_count,
                            ));
                        }
                        _ => panic!(
                            "Expected Function, found {:?}",
                            &builder.module.type_table[&air_function_type]
                        ),
                    }

                    air_arguments.extend(function_signature.ty.param_values.clone());

                    let function = Self::parse_air_function(
                        &mut builder,
                        &module,
                        function_signature.global_id,
                        &air_arguments,
                        &spirv_inputs,
                        &spirv_outputs,
                        &global_variables,
                        &constants,
                    );

                    let mut spirv_arguments = spirv_outputs.clone();
                    spirv_arguments.extend(spirv_inputs);

                    entry_points.insert(
                        function_signature.global_id,
                        builder.new_entry_point(
                            &module.string_table[function_signature.name.0 as usize].content,
                            function,
                            SpirVExecutionModel::Vertex,
                            spirv_arguments,
                        ),
                    );

                    panic!(
                        "{:?}",
                        builder
                            .module
                            .entry_point_table
                            .get(entry_points.get(&function_signature.global_id).unwrap())
                    );
                }
            }
            None => {}
        }

        todo!("{:#?}", builder.module);

        Ok(())
    }

    pub fn vec_mask_to_literal_array(air_mask: AirValueId, module: &AirModule) -> Vec<u32> {
        let vec_value = module.value_list.get(air_mask.0 as usize).unwrap();
        let vec_constant = match vec_value {
            AirValue::Constant(constant) => {
                let constant = module.constants.get(constant).unwrap();
                match &constant.value {
                    AirConstantValue::Aggregate(agg) => agg
                        .iter()
                        .map(|i| match module.value_list.get(i.0 as usize).unwrap() {
                            AirValue::Constant(c) => &module.constants.get(&c).unwrap().value,
                            _ => panic!("Expected Constant. Found {:?}", i),
                        })
                        .collect::<Vec<_>>(),
                    AirConstantValue::Array(arr) => arr.iter().map(|x| x).collect::<Vec<_>>(),
                    _ => panic!("Expected Aggregate or Array. Found {:?}", &constant.value),
                }
            }
            _ => panic!("Expected Constant. Found {:?}", vec_value),
        };

        let mut result = vec![];
        for i in vec_constant {
            match i {
                AirConstantValue::Integer(int) => result.push(*int as u32),
                AirConstantValue::Null | AirConstantValue::Undefined => result.push(0),
                _ => panic!("Expected Integer. Found {:?}", i),
            }
        }

        result
    }

    pub fn get_air_type_from_value<'a>(module: &'a AirModule, value_id: AirValueId) -> &'a AirType {
        let get_air_ty = module.value_list.get(value_id.0 as usize).unwrap();

        match get_air_ty {
            AirValue::Constant(constant) => {
                let constant = module.constants.get(constant).unwrap();
                let air_ty = module.types.get(constant.ty.0 as usize).unwrap();
                air_ty
            }
            AirValue::Load(load) => &load.ty,
            AirValue::ShuffleVec(shuffle_vec) => {
                let size = {
                    let mask = Self::get_air_type_from_value(module, shuffle_vec.mask);
                    match mask {
                        AirType::Vector(v) => v.size,
                        _ => panic!("Expected Vector, Found: {:?}", mask),
                    }
                };

                let element_type = {
                    let vec1 = Self::get_air_type_from_value(module, shuffle_vec.vec1);
                    match vec1 {
                        AirType::Vector(v) => v.element_type,
                        _ => panic!("Expected Vector, Found: {:?}", vec1),
                    }
                };

                for i in &module.types {
                    if *i == AirType::Vector(AirVectorType { size, element_type }) {
                        return i;
                    }
                }

                todo!()
            }
            AirValue::InsertVal(air_insert_val) => {
                Self::get_air_type_from_value(module, air_insert_val.value1)
            }
            _ => todo!("{:?}", get_air_ty),
        }
    }

    pub fn parse_air_value(
        builder: &mut SpirVBuilder,
        module: &AirModule,
        value_id: AirValueId,
        value_list: &HashMap<AirValueId, SpirVVariableId>,
        spirv_entry_point_outputs: &Vec<SpirVVariableId>,
    ) -> SpirVVariableId {
        let value = module.value_list.get(value_id.0 as usize).unwrap();

        match value {
            AirValue::Cast(air_cast) => {
                let to_type = Self::parse_air_type(builder, module, &air_cast.cast_to_type);

                return builder.new_bit_cast(SpirVBitCast {
                    variable: *value_list.get(&air_cast.value).unwrap(),
                    to_type,
                });
            }
            AirValue::GetElementPtr(air_gep) => {
                let element_ty = Self::parse_air_type(builder, module, &air_gep.ty);
                let pointer_ty =
                    builder.new_type(SpirVType::Pointer(SpirVStorageClass::Private, element_ty));

                let spirv_base = *value_list.get(&air_gep.base_ptr_value).unwrap();
                let mut spirv_indices = vec![];
                for i in &air_gep.indices {
                    spirv_indices.push(*value_list.get(&i).unwrap());
                }

                return builder.new_access_chain(SpirVAccessChain {
                    type_id: pointer_ty,
                    base_id: spirv_base,
                    indices: spirv_indices,
                });
            }
            AirValue::Load(air_load) => {
                let operand = value_list.get(&air_load.op).unwrap();

                let load_ty = Self::parse_air_type(builder, module, &air_load.ty);

                return builder.new_load(SpirVLoad {
                    type_id: load_ty,
                    pointer_id: *operand,
                    memory_operands: SpirVMemoryOperands::None,
                });
            }
            AirValue::ShuffleVec(air_shuffle_vec) => {
                let vec_type = Self::get_air_type_from_value(module, value_id);

                let vec1 = *value_list.get(&air_shuffle_vec.vec1).unwrap();
                let vec2 = *value_list.get(&air_shuffle_vec.vec2).unwrap();

                let mask = Self::vec_mask_to_literal_array(air_shuffle_vec.mask, module);

                let vec_type = Self::parse_air_type(builder, module, vec_type);
                builder.new_vector_shuffle(SpirVVectorShuffle {
                    vec_type,
                    vec1,
                    vec2,
                    mask,
                })
            }
            AirValue::InsertVal(air_insert_val) => {
                let result_type = Self::get_air_type_from_value(module, air_insert_val.value1);
                let result_type = Self::parse_air_type(builder, module, result_type);

                let value1 = *value_list.get(&air_insert_val.value1).unwrap();
                let value2 = *value_list.get(&air_insert_val.value2).unwrap();

                builder.new_composite_insert(SpirVCompositeInsert {
                    type_id: result_type,
                    object_id: value2,
                    composite_id: value1,
                    indices: vec![air_insert_val.insert_value_idx as u32],
                })
            }
            AirValue::Return(air_return) => {
                if spirv_entry_point_outputs.len() == 0 {
                    return builder.new_return(Some(*value_list.get(&air_return.value).unwrap()));
                }

                todo!("{:?}", builder.module.type_table.get(&SpirVVariableId(73)));

                todo!()
            }
            _ => todo!("{:?}", value),
        }
    }

    pub fn parse_air_function(
        builder: &mut SpirVBuilder,
        module: &AirModule,
        air_signature: AirFunctionSignatureId,
        air_entry_points: &Vec<AirValueId>,
        spirv_entry_point_outputs: &Vec<SpirVVariableId>,
        spirv_entry_point_inputs: &Vec<SpirVVariableId>,
        global_variables: &HashMap<AirGlobalVariableId, SpirVVariableId>,
        constants: &HashMap<AirConstantId, SpirVVariableId>,
    ) -> SpirVVariableId {
        let mut air_function_body = None;
        for i in &module.function_bodies {
            if i.signature == air_signature {
                air_function_body = Some(i);
            }
        }

        let air_function_body = air_function_body.unwrap();

        builder.new_basic_block();

        let mut value_list: HashMap<AirValueId, SpirVVariableId> = HashMap::new();

        let mut count = 0;
        for i in air_entry_points {
            value_list.insert(*i, spirv_entry_point_inputs[count]);
            count += 1;
        }

        for (id, spirv_value) in global_variables {
            value_list.insert(AirValueId(id.0), *spirv_value);
        }

        for (id, spirv_value) in constants {
            for i in 0..module.value_list.len() {
                if module.value_list[i] == AirValue::Constant(*id) {
                    value_list.insert(AirValueId(i as u64), *spirv_value);
                }
            }
        }

        for i in &air_function_body.contents {
            let value =
                Self::parse_air_value(builder, module, *i, &value_list, spirv_entry_point_outputs);

            value_list.insert(*i, value);
        }

        todo!()
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
