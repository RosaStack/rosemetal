pub mod air_parser;
pub mod llvm_bitcode;

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::air_parser::*;
    use super::llvm_bitcode::*;

    #[test]
    fn parser() -> Result<()> {
        let mut parser = Parser::new(std::fs::read("test.air")?)?;

        parser.start()?;

        Ok(())
    }

    #[test]
    fn bitstream() -> Result<()> {
        let (signature, parser) = Bitstream::from(std::fs::read("test.air")?)?;

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
