pub mod reader;

use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MtlLibraryTargetOSType {
    #[default]
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

impl MtlLibraryTargetOSType {
    pub fn from_u8(v: u8) -> Result<Self> {
        Ok(match v {
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
pub struct MtlLibraryTargetOS {
    pub ty: MtlLibraryTargetOSType,
    pub major: u16,
    pub minor: u16,
}

impl MtlLibraryTargetOS {
    pub fn from_integers(target: u8, major: u16, minor: u16) -> Result<Self> {
        Ok(Self {
            ty: MtlLibraryTargetOSType::from_u8(target)?,
            major,
            minor,
        })
    }
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum MtlLibraryPlatform {
    #[default]
    MacOS = 0x8001,
    IOS = 0x0001,
}

impl MtlLibraryPlatform {
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
pub enum MtlLibraryType {
    #[default]
    Executable,
    CoreImage,
    Dynamic,
    SymbolCompanion,
}

impl MtlLibraryType {
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
pub struct MtlLibrarySignature {
    pub target_platform: MtlLibraryPlatform,
}

#[derive(Debug, Clone, Default)]
pub struct MtlLibrary {
    pub content: Vec<u8>,
    position: usize,
}
