pub mod air_builder;
pub mod air_codegen;
pub mod air_parser;
pub mod llvm_bitcode;
pub mod metal_lib;
pub mod spirv_builder;
pub mod spirv_codegen;
pub mod spirv_parser;

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{air_codegen::AirToSpirV, metal_lib::MtlLibrary, spirv_codegen::air::SpirVToAir};

    use super::llvm_bitcode::*;

    #[test]
    fn read_metal_lib() -> Result<()> {
        let metal_lib = MtlLibrary::default().read(&std::fs::read("test-files/test.metallib")?)?;

        Ok(())
    }

    #[test]
    fn spirv_parser() -> Result<()> {
        let mut parser = super::spirv_parser::Parser::new(std::fs::read("test-files/test.spv")?);

        dbg!(parser.start()?);

        Ok(())
    }

    #[test]
    fn air_parser() -> Result<()> {
        let mut parser = super::air_parser::Parser::new(std::fs::read("test-files/test.air")?)?;

        dbg!(parser.start()?);

        Ok(())
    }

    #[test]
    fn spirv_to_air() -> Result<()> {
        let mut input = super::spirv_parser::Parser::new(std::fs::read("test-files/test.spv")?);

        let mut conversion = SpirVToAir::new(input.start()?);
        conversion.start()?;

        Ok(())
    }

    #[test]
    fn air_to_spirv() -> Result<()> {
        let mut input = super::air_parser::Parser::new(std::fs::read("test-files/test.air")?)?;

        let mut conversion = AirToSpirV::new(dbg!(input.start()?));
        conversion.start()?;

        let assembly = conversion.output.assemble_to_bytes();

        std::fs::write("result.spv", assembly)?;

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
