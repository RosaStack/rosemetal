use anyhow::{Result, anyhow};

use crate::{
    air_parser::{AirFile, AirItem, AirModule},
    spirv_builder::SpirVBuilder,
    spirv_parser::{self, SpirVCapability, SpirVOp},
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

        Ok(())
    }
}
