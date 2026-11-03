use crate::vulkan;

const VULKAN_VALIDATION_LAYERS: &std::ffi::CStr =
    unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") };

/// Holds the current vullkan context: Instance, device, etc.
pub struct VkInstance {
    #[allow(unused)]
    entry: ash::Entry,
    instance: ash::Instance,
    surface: ash::vk::SurfaceKHR,
    surface_instance: ash::khr::surface::Instance,
    physical_devices: crate::vulkan::VkPhysicalDevices,
}

impl VkInstance {
    pub fn create<D: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle>(
        display: &D,
        application_name: &str,
        application_version: crate::version::Version,
    ) -> Result<VkInstance, VkInstanceCreateError> {
        let application_name = format!("{application_name}\0");
        let application_name = std::ffi::CStr::from_bytes_with_nul(application_name.as_bytes())?;
        let application_version = application_version.to_u32();

        /* Fixme: missing terminal byte everywhere */
        let application_info = ash::vk::ApplicationInfo::default()
            .application_name(application_name)
            .application_version(application_version)
            .engine_name(crate::ENGINE_NAME)
            .api_version(crate::vulkan::VULKAN_API_VERSION);

        let entry = unsafe { ash::Entry::load()? };
        let display_handle = display
            .display_handle()
            .map_err(|error| VkInstanceCreateError::DisplayHande { error })?;
        let window_handle = display
            .window_handle()
            .map_err(|error| VkInstanceCreateError::WindowHandle { error })?;

        /* Get the extensions required by the display */
        let required_extensions = ash_window::enumerate_required_extensions(display_handle.into())?;
        let required_extensions = required_extensions.iter().cloned().collect::<Vec<_>>();

        /* Get all available layers to check support for the required layers */
        let available_layers = unsafe { entry.enumerate_instance_layer_properties() }?
            .iter()
            .map(|l| {
                /* SAFETY: The layer names returned by Vulkan shall be Ok */
                unsafe { std::ffi::CStr::from_ptr(l.layer_name.as_ptr()) }
            })
            .collect::<std::collections::HashSet<_>>();

        #[cfg(debug_assertions)]
        let mut required_layers = Vec::with_capacity(1);
        #[cfg(not(debug_assertions))]
        let required_layers: Vec<*const i8> = Vec::new();
        #[cfg(debug_assertions)]
        {
            if !available_layers.contains(VULKAN_VALIDATION_LAYERS) {
                log::warn!("Unable to load the Vulkan validation layers! No debug info will be accessible.");
            } else {
                required_layers.push(VULKAN_VALIDATION_LAYERS.as_ptr());
            }
        }

        for layer in required_layers.iter() {
            if layer.is_null() {
                log::warn!("Instance layer name pointer is null!!");
            } else {
                log::info!("Using instance layer {:#?}", unsafe { std::ffi::CStr::from_ptr(*layer) });
            }
        }

        for required_extension in required_extensions.iter() {
            if required_extension.is_null() {
                log::warn!("Instance extension name pointer is null!!");
            } else {
                log::info!("Using instance extension {:#?}", unsafe {
                    std::ffi::CStr::from_ptr(*required_extension)
                });
            }
        }

        let info = ash::vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(required_extensions.as_slice())
            .enabled_layer_names(required_layers.as_slice());

        let instance = unsafe { entry.create_instance(&info, None)? };
        let surface_instance = ash::khr::surface::Instance::new(&entry, &instance);

        log::info!("Successefuly created Vulkan instance");

        let surface =
            unsafe { ash_window::create_surface(&entry, &instance, display_handle.into(), window_handle.into(), None)? };

        let physical_devices = vulkan::physical_devices::query_physical_devices(&instance, surface, &surface_instance)?;
        physical_devices.log_all();

        Ok(VkInstance {
            entry,
            instance,
            surface,
            surface_instance,
            physical_devices,
        })
    }

    /// Get a reference to the surface of the window.
    pub fn surface(&self) -> ash::vk::SurfaceKHR {
        self.surface
    }

    pub fn surface_capabilities(
        &self,
        physical_device: &vulkan::VkPhysicalDevice,
    ) -> ash::prelude::VkResult<ash::vk::SurfaceCapabilitiesKHR> {
        let surface_instance = &self.surface_instance;
        let physical_device = physical_device.handle();
        unsafe { surface_instance.get_physical_device_surface_capabilities(physical_device, self.surface) }
    }

    /// Get the prefered physical device to run the engine on.
    ///
    /// Fixme: use player prefs to take the user prefered device, and fallback to scoring funcs
    pub fn prefered_physical_device(&self) -> Option<std::sync::Arc<crate::vulkan::VkPhysicalDevice>> {
        self.physical_devices.usable_physical_devices().first().cloned()
    }

    /// Get the names of all available physical devices, usable or not
    pub fn physical_device_names(&self) -> impl Iterator<Item = &str> {
        self.physical_devices.names()
    }
}

impl std::ops::Deref for VkInstance {
    type Target = ash::Instance;
    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl Drop for VkInstance {
    fn drop(&mut self) {
        log::info!("Destroying vulkan instance");
        unsafe {
            self.surface_instance.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

/// Errors that can cause an instance create failure.
/// This is a fatal error, as we can't communicate with the GPU without it, so we can't recover.
#[derive(Debug)]
pub enum VkInstanceCreateError {
    DisplayHande { error: raw_window_handle::HandleError },
    InvalidAppName { error: std::ffi::FromBytesWithNulError },
    Load { error: ash::LoadingError },
    Vulkan { error: ash::vk::Result },
    WindowHandle { error: raw_window_handle::HandleError },
}

impl std::fmt::Display for VkInstanceCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DisplayHande { error } => write!(f, "Failed to get the display handle: {error}"),
            Self::InvalidAppName { error } => write!(f, "Invalid application name: {error}"),
            Self::Load { error } => write!(f, "Failed to load vulkan: {error}"),
            Self::Vulkan { error } => write!(f, "Vulkan error: {error}"),
            Self::WindowHandle { error } => write!(f, "Failed to get the window handle: {error}"),
        }
    }
}

impl std::error::Error for VkInstanceCreateError {}

impl From<std::ffi::FromBytesWithNulError> for VkInstanceCreateError {
    fn from(error: std::ffi::FromBytesWithNulError) -> Self {
        Self::InvalidAppName { error }
    }
}

impl From<ash::LoadingError> for VkInstanceCreateError {
    fn from(error: ash::LoadingError) -> Self {
        Self::Load { error }
    }
}

impl From<ash::vk::Result> for VkInstanceCreateError {
    fn from(error: ash::vk::Result) -> Self {
        VkInstanceCreateError::Vulkan { error }
    }
}
