use anyhow::{Result, anyhow};

use crate::air_parser::*;

#[derive(Debug, Default)]
pub struct AirBuilder {
    current_module_id: usize,
    string_table_id: isize,
    pub file: AirFile,
}

impl AirBuilder {
    pub fn new() -> Self {
        Self {
            current_module_id: 0,
            string_table_id: -1,
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

    pub fn get_current_string_table(&mut self) -> Result<&mut AirStringTable> {
        Ok(match &mut self.file.items[self.string_table_id as usize] {
            AirItem::StringTable(string_table) => string_table,
            _ => return Err(anyhow!("No current string table found.")),
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

    pub fn new_table_string(&mut self, string: String) -> Result<TableStringId> {
        if self.string_table_id < 0 {
            self.file.items.push(AirItem::StringTable(AirStringTable {
                strings: vec![String::new()],
            }));

            self.string_table_id = self.file.items.len() as isize - 1;
        }

        let string_table = self.get_current_string_table()?;

        let begin = string_table.strings[0].len();
        let end = string.len();

        string_table.strings[0].extend(string.as_bytes().iter().map(|x| *x as char));

        let module = self.get_current_module()?;

        module.string_table.push(TableString {
            offset: begin as u64,
            size: end as u64,
            content: string,
        });

        Ok(TableStringId(module.string_table.len() as u64 - 1))
    }

    pub fn new_constant(&mut self, constant: AirConstant) -> Result<AirConstantId> {
        let module = self.get_current_module()?;
        let id = AirConstantId(module.max_constants_id);

        module.constants.insert(id, constant);
        module.value_list.push(AirValue::Constant(id));

        module.max_constants_id += 1;
        Ok(id)
    }

    pub fn new_function_signature(
        &mut self,
        name: &str,
        ty: AirFunctionType,
    ) -> Result<AirFunctionSignatureId> {
        let name = self.new_table_string(name.to_string())?;
        let module = self.get_current_module()?;
        let id = AirFunctionSignatureId(module.max_global_id);

        module.function_signatures.push(AirFunctionSignature {
            global_id: id,
            name,
            ty,
            ..Default::default()
        });

        module.max_global_id += 1;
        Ok(id)
    }

    pub fn new_global_variable(
        &mut self,
        name: &str,
        ty: AirTypeId,
        value: AirConstantId,
    ) -> Result<AirGlobalVariableId> {
        let name = self.new_table_string(name.to_string())?;
        let module = self.get_current_module()?;
        let result_id = AirGlobalVariableId(module.max_global_id);

        module.global_variables.insert(
            result_id,
            AirGlobalVariable {
                name,
                type_id: ty,
                is_const: true,
                initializer: value,
                linkage: LinkageCode::INTERNAL,
                unnamed_addr: UnnamedAddrCode::UNNAMED_ADDR,
                ..Default::default()
            },
        );

        module.max_global_id += 1;

        module.value_list.push(AirValue::GlobalVariable(result_id));

        Ok(result_id)
    }
}
