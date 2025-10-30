pub mod buffer;
pub mod command;
pub mod device;
pub mod drawable;
pub mod instance;
pub mod math;
pub mod render;
pub mod shader;
pub mod sync;

pub use buffer::*;
pub use command::*;
pub use device::*;
pub use drawable::*;
pub use instance::*;
pub use math::*;
pub use render::*;
pub use shader::*;
pub use sync::*;

#[cfg(test)]
mod tests {
    use anyhow::{Ok, Result};

    use super::*;

    #[test]
    fn headless_environment() -> Result<()> {
        let instance = RMLInstance::new(None)?;

        let _device = MTLDevice::create(instance)?;

        Ok(())
    }
}
