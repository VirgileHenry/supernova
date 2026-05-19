use crate::vulkan::utils as vk_utils;
use crate::vulkan::physical_devices::*;
use crate::vulkan::swapchain_support::SwapchainSupport;

pub const VULKAN_VALIDATION_LAYERS: [i8; 256] = vk_utils::create_vulkan_name("VK_LAYER_KHRONOS_validation");
pub const VULKAN_API_VERSION: u32 = ash::vk::make_api_version(1, 2, 0, 0);

/// Holds the current vullkan context: Instance, device, etc.
pub struct VulkanState {
    #[allow(unused)]
    entry: ash::Entry,
    instance: ash::Instance,
    pub surface: ash::vk::SurfaceKHR,
    /// Provide access to surface related functions
    pub surface_instance: ash::khr::surface::Instance,
    pub physical_device: ash::vk::PhysicalDevice,
    /// SAFETY: Must be fetched from the above physical device.
    pub queue_family_indices: crate::vulkan::QueueFamilyIndices,
    pub vulkan_interface: crate::vulkan::VulkanInterface,
}

/// We want our vulkan state to "act" as an instance for the rest of the vulkan impl.
/// 
/// This allow other objects to use our state as an instance for calling vulkan functions.
impl std::ops::Deref for VulkanState {
    type Target = ash::Instance;
    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl VulkanState {
    pub fn create<D: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle>(display: &D) -> Result<VulkanState, crate::ScError> {

        // create the app info 
        let application_info = ash::vk::ApplicationInfo {
            p_application_name: crate::consts::SPACECRAFT_NAME.as_ptr() as *const i8,
            application_version: crate::consts::SPACECRAFT_VERSION,
            p_engine_name: crate::consts::SPACECRAFT_ENGINE_NAME.as_ptr() as *const i8,
            api_version: VULKAN_API_VERSION,
            ..Default::default()
        };

        // create the vulkan loader and entry
        let entry = unsafe { ash::Entry::load()? };

        let extensions = ash_window::enumerate_required_extensions(display.display_handle().unwrap().into())?.iter().cloned().collect::<Vec<_>>();
        
        // get the validation layer
        let available_layers = unsafe {
            entry.enumerate_instance_layer_properties()?
                .iter()
                .map(|l| l.layer_name)
                .collect::<std::collections::HashSet<_>>()
        };

        #[cfg(debug_assertions)]
        let mut layers = Vec::with_capacity(1);
        #[cfg(not(debug_assertions))]
        let layers: Vec<*const i8> = Vec::with_capacity(1);
        #[cfg(debug_assertions)]
        {
            if !available_layers.contains(&VULKAN_VALIDATION_LAYERS) {
                log::warn!("Unable to load the Vulkan validation layers! No debug info will be accessible.");
            }
            else {
                layers.push(VULKAN_VALIDATION_LAYERS.as_ptr());
            }
        }

        for layer in layers.iter() {
            if layer.is_null() {
                log::warn!("Instance layer name pointer is null!!");
            }
            else {
                log::info!("Using instance layer {:#?}", unsafe { std::ffi::CStr::from_ptr(*layer) });
            }
        }
        
        for extension in extensions.iter() {
            if extension.is_null() {
                log::warn!("Instance extension name pointer is null!!");
            }
            else {
                log::info!("Using instance extension {:#?}", unsafe { std::ffi::CStr::from_ptr(*extension) });
            }
        }

        // create the vk instance info
        let info = ash::vk::InstanceCreateInfo {
            p_application_info: &application_info,
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: if extensions.is_empty() { std::ptr::null() } else { extensions.as_ptr() },
            enabled_layer_count: layers.len() as u32,
            pp_enabled_layer_names: if layers.is_empty() { std::ptr::null() } else { layers.as_ptr() },
            ..Default::default()
        };

        // create the vk instance
        let instance = unsafe { entry.create_instance(&info, None)? };
        let surface_instance = ash::khr::surface::Instance::new(&entry, &instance);

        log::info!("Successefuly created Vulkan instance");

        // create the surface : interface between vulkan and winit window.
        // let surface_instance = unsafe { ash::khr::surface::Instance::new(&entry, &instance) };
        let surface = unsafe { ash_window::create_surface(
            &entry,
            &instance,
            display.display_handle().unwrap().into(), // TODO: error handling here
            display.window_handle().unwrap().into(), // TODO: error handling here
            None
        )? };

        // get best physical device and queue family indices
        let (physical_device, queue_family_indices) = best_physical_device(&instance, &surface_instance, surface).ok_or("No Valid physical devices!")?;

        let vulkan_interface = crate::vulkan::VulkanInterface::create(&instance, physical_device, &queue_family_indices)?;
        
        match unsafe { instance.get_physical_device_properties(physical_device) }.device_name_as_c_str() {
            Ok(name) => match name.to_str() {
                Ok(name) => log::info!("Selected GPU: {}", name),
                Err(e) => log::warn!("Invalid UTF-8 encoding for selected GPU name: {e}"),
            }
            Err(e) => log::warn!("Failed to interpret GPU name as str: {e}")
        };

        Ok(VulkanState {
            entry,
            instance,
            surface,
            surface_instance,
            physical_device,
            queue_family_indices,
            vulkan_interface,
        })
    }

    pub fn current_swapchain_support(&self) -> Result<SwapchainSupport, crate::ScError> {
        SwapchainSupport::get(&self.surface_instance, self.physical_device, self.surface)
    }

}

impl Drop for VulkanState {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_interface.destroy();
            self.surface_instance.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

