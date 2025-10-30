use crate::{
    MTLBufferUsage, MTLRenderPass, MTLRenderPipelineDescriptor, MTLRenderPipelineState, MTLView,
    RMLInstance, buffer::MTLBuffer, shader::MTLLibrary,
};
use anyhow::{Result, anyhow};
#[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
use ash::vk;
#[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
use std::{
    cell::RefCell,
    collections::BTreeMap,
    sync::{RwLock, atomic::AtomicU32},
};
use std::{ffi::CStr, ptr::NonNull, sync::atomic::Ordering};
use std::{ffi::c_void, sync::Arc};

#[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
use objc2::{rc::Retained, runtime::ProtocolObject};

#[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
use objc2_metal::{MTLCreateSystemDefaultDevice, MTLDevice as MetalMTLDevice};

pub struct MTLDevice {
    name: String,
    pub instance: Arc<RMLInstance>,

    #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
    metal_device: Retained<ProtocolObject<dyn MetalMTLDevice>>,

    #[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
    vulkan_device: VulkanMTLDevice,
}

pub trait ArcMTLDevice {
    fn make_buffer<T>(&self, data: &[T], usage: MTLBufferUsage) -> Result<Arc<MTLBuffer<T>>>;
    fn new_render_pipeline_state(
        &self,
        render_pipeline_descriptor: MTLRenderPipelineDescriptor,
    ) -> Result<MTLRenderPipelineState>;

    #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
    fn metal_make_buffer<T>(&self, data: &[T]) -> Result<Arc<MTLBuffer>>;

    #[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
    fn vulkan_make_buffer<T>(&self, data: &[T], usage: MTLBufferUsage)
    -> Result<Arc<MTLBuffer<T>>>;
}

impl ArcMTLDevice for Arc<MTLDevice> {
    #[allow(unused_variables)]
    fn make_buffer<T>(&self, data: &[T], usage: MTLBufferUsage) -> Result<Arc<MTLBuffer<T>>> {
        #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
        return self.metal_make_buffer(data);

        #[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
        return self.vulkan_make_buffer(data, usage);
    }

    #[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
    fn vulkan_make_buffer<T>(
        &self,
        data: &[T],
        usage: MTLBufferUsage,
    ) -> Result<Arc<MTLBuffer<T>>> {
        let device = self.vulkan_device().logical();

        let buffer = unsafe {
            device.create_buffer(
                &vk::BufferCreateInfo::default()
                    .usage(usage.to_vulkan())
                    .size((size_of::<T>() * data.len()) as u64)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE),
                None,
            )?
        };

        todo!()
    }

    #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
    fn metal_make_buffer<T>(&self, data: &[T]) -> Result<Arc<MTLBuffer>> {
        use objc2_metal::MTLResourceOptions;

        let data_ptr = data.as_ptr();
        let data_non_null = unsafe { NonNull::new_unchecked(data_ptr as *mut c_void) };

        let metal_buffer = unsafe {
            self.metal_device().newBufferWithBytes_length_options(
                data_non_null,
                size_of::<T>() * data.len(),
                MTLResourceOptions::StorageModeShared,
            )
        }
        .unwrap();

        Ok(Arc::new(MTLBuffer::from_metal(self.clone(), metal_buffer)))
    }

    fn new_render_pipeline_state(
        &self,
        render_pipeline_descriptor: MTLRenderPipelineDescriptor,
    ) -> Result<MTLRenderPipelineState> {
        #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
        {
            return Ok(MTLRenderPipelineState::from_metal(
                self.clone(),
                render_pipeline_descriptor,
                self.metal_device()
                    .newRenderPipelineStateWithDescriptor_error(
                        &render_pipeline_descriptor.to_metal(),
                    )?,
            ));
        }

        #[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
        {
            return Ok(MTLRenderPipelineState::from_vulkan(
                self.clone(),
                render_pipeline_descriptor,
            ));
        }
    }
}

impl MTLDevice {
    pub fn create(instance: Arc<RMLInstance>) -> Result<Arc<Self>> {
        #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
        return Self::metal_create(instance);

        #[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
        return Self::vulkan_create(instance);
    }

    #[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
    pub fn vulkan_create(instance: Arc<RMLInstance>) -> Result<Arc<Self>> {
        let devices = unsafe { instance.vulkan_instance().enumerate_physical_devices()? };

        let physical_device = devices
            .into_iter()
            .find(|device| Self::vulkan_device_check(&instance, device).is_ok())
            .expect("No suitable Physical Device.");

        let queue_families = Self::vulkan_find_queue_families(&instance, &physical_device)?;

        let properties = unsafe {
            instance
                .vulkan_instance()
                .get_physical_device_properties(physical_device)
        };

        let name = unsafe {
            CStr::from_ptr(properties.device_name.as_ptr())
                .to_str()?
                .to_string()
        };

        let logical_device =
            Self::vulkan_create_logical_device(&instance, &physical_device, &queue_families)?;

        Ok(Arc::new(Self {
            name,
            instance,
            vulkan_device: VulkanMTLDevice {
                physical_device,
                logical_device,
                queue_families,
            },
        }))
    }

    #[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
    pub fn vulkan_create_logical_device(
        instance: &Arc<RMLInstance>,
        device: &vk::PhysicalDevice,
        queue_families: &VulkanQueueFamilies,
    ) -> Result<ash::Device> {
        let mut indices = vec![queue_families.graphics_queue, queue_families.present_queue];
        indices.dedup();

        let queue_info = indices
            .iter()
            .map(|index| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(*index)
                    .queue_priorities(&[1.0_f32])
            })
            .collect::<Vec<_>>();

        let device_extensions = Self::vulkan_required_extensions()
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_info)
            .enabled_extension_names(&device_extensions);

        Ok(unsafe {
            instance
                .vulkan_instance()
                .create_device(*device, &device_create_info, None)?
        })
    }

    #[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
    pub fn vulkan_device_check(
        instance: &Arc<RMLInstance>,
        device: &vk::PhysicalDevice,
    ) -> Result<()> {
        Self::vulkan_find_queue_families(instance, device)?;

        Self::vulkan_check_extension_support(instance, device)?;

        Ok(())
    }

    #[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
    pub fn vulkan_find_queue_families(
        instance: &Arc<RMLInstance>,
        device: &vk::PhysicalDevice,
    ) -> Result<VulkanQueueFamilies> {
        let mut graphics: Option<u32> = None;
        let mut present: Option<u32> = None;

        let properties = unsafe {
            instance
                .vulkan_instance()
                .get_physical_device_queue_family_properties(*device)
        };

        for (index, family) in properties.iter().filter(|f| f.queue_count > 0).enumerate() {
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) && graphics.is_none() {
                graphics = Some(index as u32);
            }

            // TODO: Present Support so far has to be done with a surface.
            // Find a way to get present support without it.
            //
            // By finding out, we could also follow the Metal way of requesting
            // a surface without having to instantiate it in `BMLInstance`.
            let present_support = unsafe {
                match instance.vulkan_surface() {
                    Some(v) => v.instance().get_physical_device_surface_support(
                        *device,
                        index as u32,
                        *v.khr(),
                    )?,
                    // Fallback to the graphics queue.
                    // Not ideal, but it'll work for now...
                    None => graphics.is_some(),
                }
            };

            if present_support && present.is_none() {
                present = Some(index as u32);
            }

            if graphics.is_some() && present.is_some() {
                break;
            }
        }

        if graphics.is_some() && present.is_some() {
            return Ok(VulkanQueueFamilies {
                graphics_queue: graphics.unwrap(),
                present_queue: present.unwrap(),
            });
        }

        Err(anyhow!("No suitable Graphics and Present queue found."))
    }

    #[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
    pub fn vulkan_required_extensions() -> Vec<&'static CStr> {
        if cfg!(any(target_os = "macos", target_os = "ios")) {
            return vec![
                ash::khr::swapchain::NAME,
                ash::khr::portability_subset::NAME,
            ];
        }

        vec![ash::khr::swapchain::NAME]
    }

    #[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
    pub fn vulkan_check_extension_support(
        instance: &Arc<RMLInstance>,
        device: &vk::PhysicalDevice,
    ) -> Result<()> {
        let required_extensions = Self::vulkan_required_extensions();

        let extension_properties = unsafe {
            instance
                .vulkan_instance()
                .enumerate_device_extension_properties(*device)?
        };

        for required in required_extensions.iter() {
            let found = extension_properties.iter().any(|ext| {
                let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                required == &name
            });

            if !found {
                return Err(anyhow!("No required extension found."));
            }
        }

        Ok(())
    }

    #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
    pub fn metal_create(instance: Arc<RMLInstance>) -> Result<Arc<Self>> {
        let metal_device = MTLCreateSystemDefaultDevice();

        let metal_device = match metal_device {
            Some(m) => m,
            None => return Err(anyhow!("No device found.")),
        };

        let name = metal_device.name().to_string();

        Ok(Arc::new(Self {
            name,
            instance,
            metal_device,
        }))
    }

    #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
    pub fn metal_device(&self) -> &Retained<ProtocolObject<dyn MetalMTLDevice>> {
        &self.metal_device
    }

    #[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
    pub fn vulkan_device(&self) -> &VulkanMTLDevice {
        &self.vulkan_device
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn new_library(&self, content: &[u8]) -> Result<Arc<MTLLibrary>> {
        #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
        return self.metal_new_library(content);

        #[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
        todo!("Vulkan Support.")
    }

    #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
    pub fn metal_new_library(&self, content: &[u8]) -> Result<Arc<MTLLibrary>> {
        Ok(Arc::new(MTLLibrary::from_metal_lib(content, self)?))
    }
}

#[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
pub struct VulkanMTLDevice {
    physical_device: vk::PhysicalDevice,
    logical_device: ash::Device,
    queue_families: VulkanQueueFamilies,
}

#[cfg(any(not(any(target_os = "macos", target_os = "ios")), feature = "moltenvk"))]
impl VulkanMTLDevice {
    pub fn physical(&self) -> &vk::PhysicalDevice {
        &self.physical_device
    }

    pub fn logical(&self) -> &ash::Device {
        &self.logical_device
    }

    pub fn queue_families(&self) -> &VulkanQueueFamilies {
        &self.queue_families
    }
}

pub struct VulkanQueueFamilies {
    pub graphics_queue: u32,
    pub present_queue: u32,
}
