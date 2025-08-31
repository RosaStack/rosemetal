use anyhow::{Result, anyhow};

use crate::air_parser::*;

#[derive(Debug, Default)]
pub struct AirBuilder {
    current_module_id: usize,
    pub file: AirFile,
}

impl AirBuilder {
    pub fn new() -> Self {
        Self {
            current_module_id: 0,
            file: AirFile::default(),
        }
    }

    pub fn identification(&mut self, string: &str) {
        self.file
            .items
            .push(AirItem::IdentificationBlock(AirIdentificationBlock {
                string: string.to_string(),
                epoch: vec![],
            }));
    }

    pub fn get_current_module(&mut self) -> Result<&mut AirModule> {
        Ok(match &mut self.file.items[self.current_module_id] {
            AirItem::Module(module) => module,
            _ => return Err(anyhow!("No current module selected or found.")),
        })
    }

    pub fn apple_ir_data_layout() -> &'static str {
        "e-p:64:64:64-i1:8:8-i8:8:8-i16:16:16-i32:32:32-i64:64:64-f32:32:32-f64:64:64-v16:16:16-v24:32:32-v32:32:32-v48:64:64-v64:64:64-v96:128:128-v128:128:128-v192:256:256-v256:256:256-v512:512:512-v1024:1024:1024-n8:16:32"
    }

    pub fn begin_apple_shader_module(&mut self, filename: &str) -> Result<()> {
        self.begin_module(
            filename,
            12,
            &["air64", "apple", "macosx15.0.0"],
            Self::apple_ir_data_layout(),
        )
    }

    pub fn begin_module(
        &mut self,
        source_filename: &str,
        version: u64,
        triple: &[&'static str],
        data_layout: &str,
    ) -> Result<()> {
        self.file.items.push(AirItem::Module(AirModule::default()));
        self.current_module_id = self.file.items.len() - 1;

        let module = self.get_current_module()?;
        module.version = version;

        module.use_relative_ids = version >= 1;

        module.triple = triple.join("-");
        module.data_layout = data_layout.to_string();
        module.source_filename = source_filename.to_string();

        Ok(())
    }

    pub fn new_type(&mut self, ty: AirType) -> Result<AirTypeId> {
        let module = self.get_current_module()?;

        module.types.push(ty);

        Ok(AirTypeId(module.types.len() as u64 - 1))
    }
}
