use std::ffi::{CStr, CString};

use airlines::metal_lib::MTLLibraryParser;
use anyhow::Result;

#[cfg(all(any(target_os = "macos", target_os = "ios"), feature = "moltenvk"))]
use ash::vk;

#[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
use objc2::{rc::Retained, runtime::ProtocolObject};

#[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
use objc2_metal::{MTLFunction as MetalMTLFunction, MTLLibrary as MetalMTLLibrary};

use crate::MTLDevice;

pub struct MTLLibrary {
    parser: MTLLibraryParser,

    #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
    native_mtl_library: Retained<ProtocolObject<dyn MetalMTLLibrary>>,

    #[cfg(all(any(target_os = "macos", target_os = "ios"), feature = "moltenvk"))]
    vulkan_shader_module: vk::ShaderModule,
}

impl MTLLibrary {
    pub fn from_metal_lib(content: &[u8], device: &MTLDevice) -> Result<Self> {
        let parser = MTLLibraryParser::default().read(content)?.clone();

        #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
        {
            use objc2_metal::MTLDevice;

            let data = dispatch2::DispatchData::from_bytes(content);
            let native_mtl_library = unsafe {
                device
                    .metal_device()
                    .newLibraryWithData_error(&data)
                    .unwrap()
            };

            return Ok(Self {
                parser,
                native_mtl_library,
            });
        }

        #[cfg(all(any(target_os = "macos", target_os = "ios"), feature = "moltenvk"))]
        {
            let spirv_result = parser.to_spirv_binary();

            let vulkan_shader_module = unsafe {
                device.vulkan_device().logical().create_shader_module(
                    &vk::ShaderModuleCreateInfo::default().code(&spirv_result),
                    None,
                )?
            };

            return Ok(Self {
                parser,
                vulkan_shader_module,
            });
        }
    }

    pub fn get_function(&self, name: &str, function_type: MTLFunctionType) -> Result<MTLFunction> {
        #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
        {
            use objc2_foundation::NSString;

            let string = NSString::from_str(name);
            let native_mtl_function = self
                .native_mtl_library
                .newFunctionWithName(string.downcast_ref().unwrap())
                .unwrap();

            return Ok(MTLFunction::from_metal(
                native_mtl_function,
                function_type,
                name.to_string(),
            ));
        }

        return Ok(MTLFunction::from_vulkan(
            function_type,
            self.vulkan_shader_module,
            name.to_string(),
        ));
    }
}

#[derive(Clone)]
pub enum MTLFunctionType {
    Vertex,
    Fragment,
}

#[derive(Clone)]
pub struct MTLFunction {
    name: String,
    c_string_name: CString,
    function_type: MTLFunctionType,
    vulkan_shader_module: vk::ShaderModule,

    #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
    native_mtl_function: Retained<ProtocolObject<dyn MetalMTLFunction>>,
}

impl MTLFunction {
    #[cfg(all(any(target_os = "macos", target_os = "ios"), feature = "moltenvk"))]
    pub fn from_vulkan(
        function_type: MTLFunctionType,
        vulkan_shader_module: vk::ShaderModule,
        name: String,
    ) -> Self {
        Self {
            name: name.clone(),
            c_string_name: CString::new(name).unwrap(),
            vulkan_shader_module,
            function_type,
        }
    }

    #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
    pub fn from_metal(
        native_mtl_function: Retained<ProtocolObject<dyn MetalMTLFunction>>,
        function_type: MTLFunctionType,
        name: String,
    ) -> Self {
        Self {
            name: name.clone(),
            c_string_name: CString::new(name).unwrap(),
            function_type,
            native_mtl_function,
        }
    }

    #[cfg(all(any(target_os = "macos", target_os = "ios"), not(feature = "moltenvk")))]
    pub fn to_metal(&self) -> &Retained<ProtocolObject<dyn MetalMTLFunction>> {
        &self.native_mtl_function
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn c_string_name(&self) -> &CStr {
        &self.c_string_name
    }

    pub fn function_type(&self) -> &MTLFunctionType {
        &self.function_type
    }

    #[cfg(all(any(target_os = "macos", target_os = "ios"), feature = "moltenvk"))]
    pub fn vulkan_shader_module(&self) -> &vk::ShaderModule {
        &self.vulkan_shader_module
    }

    #[cfg(all(any(target_os = "macos", target_os = "ios"), feature = "moltenvk"))]
    pub fn vulkan_pipeline_stage_create_info(&self) -> vk::PipelineShaderStageCreateInfo {
        vk::PipelineShaderStageCreateInfo::default()
            .stage(match self.function_type {
                MTLFunctionType::Vertex => vk::ShaderStageFlags::VERTEX,
                MTLFunctionType::Fragment => vk::ShaderStageFlags::FRAGMENT,
            })
            .module(*self.vulkan_shader_module())
            .name(self.c_string_name())
    }
}
