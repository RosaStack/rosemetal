pub mod air_builder;
pub mod air_parser;
pub mod llvm_bitcode;
pub mod spirv_parser;

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{air_builder::AirBuilder, air_parser::AirType};

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

        let void_type = builder.new_type(AirType::Void)?;

        dbg!("{:?}", builder.file);

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
