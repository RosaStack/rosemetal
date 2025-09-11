use std::collections::HashMap;

use anyhow::{Result, anyhow};

use crate::{
    air_parser::{
        AirConstant, AirConstantId, AirConstantValue, AirFile, AirFunctionSignature,
        AirFunctionSignatureId, AirGlobalVariableId, AirItem, AirMetadataConstant,
        AirMetadataNamedNode, AirModule, AirType, AirTypeId, AirValue,
    },
    spirv_builder::SpirVBuilder,
    spirv_parser::{
        SpirVAddressingModel, SpirVCapability, SpirVConstant, SpirVConstantComposite,
        SpirVConstantValue, SpirVEntryPoint, SpirVMemoryModel, SpirVOp, SpirVStorageClass,
        SpirVType, SpirVVariableId,
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
        id: AirTypeId,
    ) -> SpirVVariableId {
        match &module.types[id.0 as usize] {
            AirType::Void => builder.new_type(SpirVType::Void),
            AirType::Integer(width) => builder.new_type(SpirVType::Int(*width as u32, false)),
            AirType::Float => builder.new_type(SpirVType::Float(32)),
            AirType::Function(function_ty) => {
                let return_ty = Self::parse_air_type(builder, module, function_ty.return_type);
                let args = function_ty
                    .params
                    .iter()
                    .map(|ty| Self::parse_air_type(builder, module, *ty))
                    .collect::<Vec<_>>();

                builder.new_type(SpirVType::Function(return_ty, args))
            }
            AirType::Struct(struct_ty) => {
                let elements = struct_ty
                    .elements
                    .iter()
                    .map(|ty| (String::new(), Self::parse_air_type(builder, module, *ty)))
                    .collect::<Vec<_>>();

                builder.new_struct_type(&struct_ty.name, struct_ty.name.is_empty(), elements)
            }
            AirType::Array(array_ty) => {
                let element_ty = Self::parse_air_type(builder, module, array_ty.element_type);
                builder.new_type(SpirVType::Array(element_ty, array_ty.size as u32))
            }
            AirType::Vector(vector_ty) => {
                let element_ty = Self::parse_air_type(builder, module, vector_ty.element_type);
                builder.new_type(SpirVType::Vector(element_ty, vector_ty.size as u32))
            }
            _ => todo!("{:?}", &module.types[id.0 as usize]),
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

        let mut module = module.unwrap();
        let mut builder = SpirVBuilder::new();

        builder.set_version(1, 0);
        builder.add_capability(SpirVCapability::Shader);
        builder.add_memory_model(SpirVAddressingModel::Logical, SpirVMemoryModel::Glsl450);

        let mut constants: HashMap<AirConstantId, SpirVVariableId> = HashMap::new();
        for (id, constant) in &module.constants {
            let ty = Self::parse_air_type(&mut builder, &module, constant.ty);
            let constant =
                Self::parse_air_constant(&mut builder, &module, ty, Some(constant.clone()), None);
            constants.insert(*id, constant);
        }

        let mut global_variables: HashMap<AirGlobalVariableId, SpirVVariableId> = HashMap::new();
        for (id, global_var) in &module.global_variables {
            let ty = Self::parse_air_type(&mut builder, &module, global_var.type_id);
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

                    let entry_point_values = match module.metadata_constants.get(&entry[1]).unwrap()
                    {
                        AirMetadataConstant::Node(entry_point_values) => entry_point_values,
                        _ => {
                            panic!(
                                "Expected Node Group, found {:?}",
                                module.metadata_constants[&entry[1]]
                            )
                        }
                    };

                    let entry_point_vertex_id =
                        match module.metadata_constants.get(&entry[2]).unwrap() {
                            AirMetadataConstant::Node(entry_point_vertex_id) => {
                                entry_point_vertex_id
                            }
                            _ => {
                                panic!(
                                    "Expected Node Group, found {:?}",
                                    module.metadata_constants[&entry[1]]
                                )
                            }
                        };

                    let vertex_info = Self::parse_vertex_info(
                        &module,
                        entry_point_values.clone(),
                        entry_point_vertex_id.clone(),
                    );
                }
            }
            None => {}
        }

        todo!("{:#?}", builder.module);

        Ok(())
    }

    pub fn parse_vertex_info(
        module: &AirModule,
        vertex_values_info: Vec<u64>,
        vertex_id_info: Vec<u64>,
    ) -> VertexFunctionInfo {
        let mut vertex_outputs: Vec<VertexOutput> = vec![];
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

            let mut count = 0;
            loop {
                let variable_name = module
                    .get_metadata_string(vertex_properties[count])
                    .unwrap();

                match variable_name.as_str() {
                    "air.vertex_output" => {
                        let mut vertex_output = VertexOutput::default();
                        count += 1;
                        let user_location_string = module
                            .get_metadata_string(vertex_properties[count])
                            .unwrap();

                        let start_location_num =
                            user_location_string.find("user(locn").unwrap() + "user(locn".len();
                        let end_location_num = user_location_string.find(")").unwrap();
                        let user_location_string =
                            &user_location_string[start_location_num..end_location_num].to_string();

                        vertex_output.location = user_location_string.parse::<u64>().unwrap();
                    }
                    _ => todo!(),
                }

                count += 1;
            }
        }

        todo!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct VertexOutput {
    pub name: String,
    pub ty: String,
    pub location: u64,
}

#[derive(Debug, Default, Clone)]
pub struct VertexFunctionInfo {
    pub vertex_outputs: Vec<VertexOutput>,
}
