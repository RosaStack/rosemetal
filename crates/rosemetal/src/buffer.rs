use std::{marker::PhantomData, sync::Arc};

#[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
use ash::vk;
#[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
use objc2::{rc::Retained, runtime::ProtocolObject};
pub use objc2_metal::MTLBuffer as MetalMTLBuffer;

use crate::MTLDevice;

#[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
pub struct VulkanMTLBuffer {
    buffer: vk::Buffer,
    device_memory: vk::DeviceMemory,
}

pub enum MTLBufferUsage {
    Vertex,
}

impl MTLBufferUsage {
    #[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
    pub fn to_vulkan(&self) -> vk::BufferUsageFlags {
        match self {
            Self::Vertex => vk::BufferUsageFlags::VERTEX_BUFFER,
        }
    }
}

pub struct MTLBuffer<T> {
    device: Arc<MTLDevice>,
    phantom: PhantomData<T>,
    #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
    metal_buffer: Retained<ProtocolObject<dyn MetalMTLBuffer>>,

    #[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
    vulkan_buffer: VulkanMTLBuffer,
}

impl<T> MTLBuffer<T> {
    #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
    pub fn from_metal<T>(
        device: Arc<MTLDevice>,
        metal_buffer: Retained<ProtocolObject<dyn MetalMTLBuffer>>,
    ) -> Self {
        Self {
            device,
            metal_buffer,
        }
    }

    #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
    pub fn metal_buffer(&self) -> &Retained<ProtocolObject<dyn MetalMTLBuffer>> {
        &self.metal_buffer
    }
}
