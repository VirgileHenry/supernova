/// Surface support capabilities of a given physical device.
#[derive(Debug, Clone)]
pub struct SurfaceSupport {
    /// Supported formats for the surface regarding the physical device.
    formats: Vec<ash::vk::SurfaceFormatKHR>,
    /// Present modes for the surface regarding the physical device.
    present_modes: Vec<ash::vk::PresentModeKHR>,
}

impl SurfaceSupport {
    pub fn get(
        surface: ash::vk::SurfaceKHR,
        surface_instance: &ash::khr::surface::Instance,
        physical_device: ash::vk::PhysicalDevice,
    ) -> ash::prelude::VkResult<Self> {
        Ok(Self {
            formats: unsafe { surface_instance.get_physical_device_surface_formats(physical_device, surface)? },
            present_modes: unsafe { surface_instance.get_physical_device_surface_present_modes(physical_device, surface)? },
        })
    }

    pub fn formats(&self) -> &[ash::vk::SurfaceFormatKHR] {
        self.formats.as_slice()
    }

    pub fn present_modes(&self) -> &[ash::vk::PresentModeKHR] {
        self.present_modes.as_slice()
    }
}
