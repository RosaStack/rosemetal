use std::{collections::HashMap, hash::Hash};

use anyhow::Result;

use crate::{
    air_builder::AirBuilder,
    air_parser::{AirArrayType, AirFunctionType, AirStructType, AirType, AirTypeId, AirVectorType},
    spirv_parser::{self, SpirVModule, SpirVOp, SpirVType, SpirVVariableId},
};

pub struct SpirVToAir {
    pub input: SpirVModule,
    pub output: AirBuilder,
}

impl SpirVToAir {
    pub fn new(input: SpirVModule) -> Result<Self> {
        Ok(Self {
            input,
            output: AirBuilder::new(),
        })
    }

    pub fn start(&mut self) -> Result<()> {
        self.output.identification("Airlines SPIR-V Shader.");

        self.output.begin_apple_shader_module("test.spv")?;

        let mut spirv_types = self
            .input
            .type_table
            .clone()
            .into_iter()
            .collect::<Vec<_>>();
        spirv_types.sort_by(|x, y| x.0.cmp(&y.0));

        let mut spirv_entry_points = self
            .input
            .entry_point_table
            .clone()
            .into_iter()
            .collect::<Vec<_>>();
        spirv_entry_points.sort_by(|x, y| x.0.cmp(&y.0));

        let mut types: HashMap<SpirVVariableId, AirTypeId> = HashMap::new();

        for (id, ty) in &spirv_types {
            types.insert(
                *id,
                match ty {
                    SpirVType::Float(_) => self.output.new_type(AirType::Float)?,
                    SpirVType::Int(width, _) => {
                        self.output.new_type(AirType::Integer(*width as u64))?
                    }
                    SpirVType::Void => self.output.new_type(AirType::Void)?,
                    SpirVType::Array(type_id, size) => {
                        self.output.new_type(AirType::Array(AirArrayType {
                            size: *size as u64,
                            element_type: *types.get(type_id).unwrap(),
                        }))?
                    }
                    SpirVType::Vector(type_id, size) => {
                        self.output.new_type(AirType::Vector(AirVectorType {
                            size: *size as u64,
                            element_type: *types.get(type_id).unwrap(),
                        }))?
                    }
                    SpirVType::Pointer(_, type_id) => self
                        .output
                        .new_type(AirType::Pointer(0, *types.get(type_id).unwrap()))?,
                    SpirVType::Function(id) => {
                        self.output.new_type(AirType::Function(AirFunctionType {
                            vararg: 0,
                            return_type: *types.get(id).unwrap(),
                            params: vec![],
                        }))?
                    }
                    SpirVType::Struct(variables) => {
                        let name = match self.input.name_table.get(id) {
                            Some(spirv_name) => spirv_name.name.clone(),
                            None => "".to_string(),
                        };

                        let is_packed = name.is_empty();

                        self.output.new_type(AirType::Struct(AirStructType {
                            name,
                            is_packed,
                            elements: variables
                                .iter()
                                .map(|x| *types.get(x).unwrap())
                                .collect::<Vec<_>>(),
                        }))?
                    }
                    _ => todo!("{:?}", ty),
                },
            );
        }

        todo!()
    }
}
