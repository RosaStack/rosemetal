pub mod reader;

use anyhow::{Result, anyhow};

use crate::{air_codegen::AirToSpirV, air_parser::AirFile};

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MTLLibraryTargetOSType {
    #[default]
    Unknown = 0,
    MacOS = 0x81,
    IOS = 0x82,
    TvOS = 0x83,
    WatchOS = 0x84,
    BridgeOS = 0x85,
    MacCatalyst = 0x86,
    IOSSimulator = 0x87,
    TvOSSimulator = 0x88,
    WatchOSSimulator = 0x89,
}

impl MTLLibraryTargetOSType {
    pub fn from_u8(v: u8) -> Result<Self> {
        Ok(match v {
            0 => Self::Unknown,
            0x81 => Self::MacOS,
            0x82 => Self::IOS,
            0x83 => Self::TvOS,
            0x84 => Self::WatchOS,
            0x85 => Self::BridgeOS,
            0x86 => Self::MacCatalyst,
            0x87 => Self::IOSSimulator,
            0x88 => Self::TvOSSimulator,
            0x89 => Self::WatchOSSimulator,
            _ => return Err(anyhow!("Invalid Target OS")),
        })
    }
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
pub struct MTLLibraryTargetOS {
    pub ty: MTLLibraryTargetOSType,
    pub major: u16,
    pub minor: u16,
}

impl MTLLibraryTargetOS {
    pub fn new(ty: MTLLibraryTargetOSType, major: u16, minor: u16) -> Self {
        Self { ty, major, minor }
    }
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum MTLLibraryPlatform {
    #[default]
    MacOS = 0x8001,
    IOS = 0x0001,
}

impl MTLLibraryPlatform {
    pub fn from_u16(v: u16) -> Result<Self> {
        Ok(match v {
            0x8001 => Self::MacOS,
            0x0001 => Self::IOS,
            _ => return Err(anyhow!("Unexpected Metal Target Platform.")),
        })
    }
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MTLLibraryType {
    #[default]
    Executable,
    CoreImage,
    Dynamic,
    SymbolCompanion,
}

impl MTLLibraryType {
    pub fn from_u8(v: u8) -> Result<Self> {
        Ok(match v {
            0 => Self::Executable,
            1 => Self::CoreImage,
            2 => Self::Dynamic,
            3 => Self::SymbolCompanion,
            _ => return Err(anyhow!("Invalid Metal Library Type.")),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct MTLLibrarySignature {
    pub target_platform: MTLLibraryPlatform,
    pub version: (u16, u16),
    pub library_type: MTLLibraryType,
    pub target_os: MTLLibraryTargetOS,
    pub file_size: u64,
    pub function_list_offset: u64,
    pub function_list_size: u64,
    pub public_metadata_offset: u64,
    pub public_metadata_size: u64,
    pub private_metadata_offset: u64,
    pub private_metadata_size: u64,
    pub bitcode_offset: u64,
    pub bitcode_size: u64,
}

#[derive(Debug, Clone, Default)]
pub struct RMLShader {
    pub air: Option<AirFile>,
}

impl RMLShader {
    pub fn from_air_file(content: AirFile) -> Self {
        Self { air: Some(content) }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MTLLibraryParser {
    pub content: Vec<u8>,
    pub signature: MTLLibrarySignature,
    pub shader: RMLShader,
    position: usize,
}

impl MTLLibraryParser {
    pub fn to_spirv_binary(&self) -> Vec<u32> {
        let mut air_to_spirv = AirToSpirV::new(self.shader.air.clone().unwrap());

        air_to_spirv.start().unwrap();

        air_to_spirv.output.assemble()
    }
}
