use crate::spirv_parser::{SpirVCapability, SpirVModule};

pub struct SpirVBuilder {
    pub module: SpirVModule,
}

impl SpirVBuilder {
    pub fn new() -> Self {
        Self {
            module: SpirVModule::default(),
        }
    }

    pub fn set_version(&mut self, major: u8, minor: u8) {
        self.module.signature.version = (major, minor);
    }

    pub fn add_capability(&mut self, capability: SpirVCapability) {
        self.module.capabilities.push(capability);
    }
}
