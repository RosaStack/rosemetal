use objc2_metal::MTLPrimitiveType as MetalMTLPrimitiveType;

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct MTLFloat3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl MTLFloat3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

pub enum MTLPrimitiveType {
    Triangle,
}

impl MTLPrimitiveType {
    #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
    pub fn to_metal(&self) -> MetalMTLPrimitiveType {
        match self {
            Self::Triangle => MetalMTLPrimitiveType::Triangle,
        }
    }
}
