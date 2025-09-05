pub mod air_builder;
pub mod air_parser;
pub mod llvm_bitcode;
pub mod spirv_parser;

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{
        air_builder::AirBuilder,
        air_parser::{AirArrayType, AirConstant, AirConstantValue, AirFunctionType, AirType},
    };

    use super::llvm_bitcode::*;

    #[test]
    fn spirv_parser() -> Result<()> {
        let mut parser = super::spirv_parser::Parser::new(std::fs::read("test-files/test.spv")?);

        parser.start()?;

        dbg!(&parser.operands);

        Ok(())
    }

    #[test]
    fn air_parser() -> Result<()> {
        let mut parser = super::air_parser::Parser::new(std::fs::read("test-files/test.air")?)?;

        dbg!(parser.start()?);

        Ok(())
    }

    #[test]
    fn air_builder() -> Result<()> {
        let mut builder = AirBuilder::new();

        builder.identification("Airlines Testing.");

        builder.begin_apple_shader_module("test.metal")?;

        let float = builder.new_type(AirType::Float)?;
        let _float_array = builder.new_type(AirType::Array(AirArrayType {
            size: 3,
            element_type: float,
        }));

        let float_const = builder.new_constant(AirConstant {
            ty: float,
            value: AirConstantValue::Float32(0_f32),
        })?;

        let value_one = builder.new_global_variable("first_variable", float, float_const)?;
        let value_two = builder.new_global_variable("second_variable", float, float_const)?;

        let function_signature = builder.new_function_signature(
            "main",
            AirFunctionType {
                vararg: 0,
                return_type: float,
                params: vec![float],
            },
        )?;
        // let function = builder.new_function(function_signature, &[]);

        dbg!(builder.file);

        Ok(())
    }

    #[test]
    fn air_bitstream() -> Result<()> {
        let (_signature, parser) = Bitstream::from(std::fs::read("test-files/test.air")?)?;

        let mut scope = 0;
        for entry in parser {
            match entry? {
                StreamEntry::SubBlock(block) => {
                    println!("{}BLOCK {} {{", "\t".repeat(scope), block.block_id);
                    scope += 1;
                }
                StreamEntry::EndBlock => {
                    scope -= 1;
                    println!("{}}}", "\t".repeat(scope));
                }
                StreamEntry::Record(record) => {
                    println!(
                        "{}RECORD {{ code: {}, fields: {:?} }}",
                        "\t".repeat(scope),
                        record.code,
                        record.fields
                    )
                }
                _ => {
                    scope -= 1;
                }
            };
        }

        Ok(())
    }
}
