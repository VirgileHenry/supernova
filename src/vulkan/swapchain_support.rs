pub struct SwapchainSupport {
    pub capabilities: ash::vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<ash::vk::SurfaceFormatKHR>,
    pub present_modes: Vec<ash::vk::PresentModeKHR>
}

impl SwapchainSupport {
    pub(super) fn get(surface_instance: &ash::khr::surface::Instance, physical_device: ash::vk::PhysicalDevice, surface: ash::vk::SurfaceKHR) -> Result<SwapchainSupport, crate::ScError> {
        Ok(SwapchainSupport {
            capabilities: unsafe {
                surface_instance.get_physical_device_surface_capabilities(physical_device, surface)?
            },
            formats: unsafe {
                surface_instance.get_physical_device_surface_formats(physical_device, surface)?
            },
            present_modes: unsafe {
                surface_instance.get_physical_device_surface_present_modes(physical_device, surface)?
            },
        })
    }

    pub(super) fn suitable(&self) -> bool {
        !self.formats.is_empty() && !self.present_modes.is_empty()
    }
}

