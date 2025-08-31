pub mod air_parser;
pub mod llvm_bitcode;
pub mod spirv_parser;

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::air_parser::*;
    use super::llvm_bitcode::*;
    use super::spirv_parser::*;

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
