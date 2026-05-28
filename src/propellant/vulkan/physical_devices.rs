mod requirements;
mod surface_support;

pub use requirements::required_device_extensions;
pub use requirements::required_device_features;
pub use surface_support::SurfaceSupport;

use crate::propellant::vulkan;

/// Cached info about a vulkan physical device.
///
/// These are meant to be fetched once at application startup,
/// then stored and queried if required.
///
/// If this device is constructed, it meets all the minimum requirements and can be used.
///
/// These informations are true for a given `(instance, surface)` pair.
/// If the surface ever gets invalidated, we need to throw these info and rebuild them.
#[derive(Clone)]
pub struct VkPhysicalDevice {
    /// Display name of the physical device.
    name: String,
    /// Display name of the driver.
    driver: String,

    /// The raw handle of the physical device.
    /// This is queried from the vulkan instance.
    handle: ash::vk::PhysicalDevice,

    /// Cached properties of the physical device.
    properties: ash::vk::PhysicalDeviceProperties,
    /// Cached properties for 1.2 of the physical device.
    properties_1_2: ash::vk::PhysicalDeviceVulkan12Properties<'static>, /* Fixme: remove the static, own the data ? that means copying the structs  */
    /// Cached properties for 1.3 of the physical device.
    properties_1_3: ash::vk::PhysicalDeviceVulkan13Properties<'static>, /* Fixme: remove the static, own the data ? that means copying the structs  */

    /// Cached memory properties of the physical device.
    memory_properties: ash::vk::PhysicalDeviceMemoryProperties,

    /// Cached standard features of the physical device.
    features: ash::vk::PhysicalDeviceFeatures,
    /// Cached features for 1.2 of the physical device.
    features_1_2: ash::vk::PhysicalDeviceVulkan12Features<'static>, /* Fixme: remove the static, own the data ? that means copying the structs  */
    /// Cached features for 1.3 of the physical device.
    features_1_3: ash::vk::PhysicalDeviceVulkan13Features<'static>, /* Fixme: remove the static, own the data ? that means copying the structs  */

    /// Available queue families on this device.
    /// Indexed by queue family index.
    queue_families: Vec<ash::vk::QueueFamilyProperties>,

    /// Extensions supported by the physical device.
    supported_extensions: std::collections::BTreeSet<String>,

    /// Surface-dependent capabilities for OUR surface.
    surface_support: SurfaceSupport,

    /// Pre-computed queue family assignments based on our needs.
    queue_family_indices: vulkan::QueueFamilyIndices,
}

impl VkPhysicalDevice {
    pub fn handle(&self) -> ash::vk::PhysicalDevice {
        self.handle
    }

    pub fn memory_properties(&self) -> &ash::vk::PhysicalDeviceMemoryProperties {
        &self.memory_properties
    }

    pub fn queue_family_indices(&self) -> vulkan::QueueFamilyIndices {
        self.queue_family_indices
    }

    pub fn surface_support(&self) -> &SurfaceSupport {
        &self.surface_support
    }
}

impl std::fmt::Debug for VkPhysicalDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {} ({:?})", self.name, self.driver, self.handle)
    }
}

impl std::fmt::Display for VkPhysicalDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.name, self.driver)
    }
}

/// Cached info about a vulkan physical device.
///
/// This version is for physical devices that does not
/// meet the minimum requirements to be used by the engine.
///
/// The struct acts as a placeholder with the minimum informations
/// so we still know that the device exists.
pub struct VkUnusablePhysicalDevice {
    /// Display name of the physical device.
    name: String,
    /// Display name of the driver.
    driver: String,

    /// The raw handle of the physical device.
    /// This is queried from the vulkan instance.
    handle: ash::vk::PhysicalDevice,

    /// Reason on to why the physical device can't be used.
    reason: String,
}

impl std::fmt::Debug for VkUnusablePhysicalDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {} ({:?})", self.name, self.driver, self.handle)
    }
}

impl std::fmt::Display for VkUnusablePhysicalDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.name, self.driver)
    }
}

/// All available physical devices.
///
/// This structure split physical devices into two categories:
/// - `usable_physical_devices` that met the required criterion to be used for the engine
/// - `unusable_physical_devices` that did not met those criterion, and can't be used.
pub struct VkPhysicalDevices {
    /// List of all usable physical devices.
    usable_physical_devices: Vec<std::sync::Arc<VkPhysicalDevice>>,
    /// List of all unusable physical devices.
    unusable_physical_devices: Vec<std::sync::Arc<VkUnusablePhysicalDevice>>,
}

impl VkPhysicalDevices {
    pub fn usable_physical_devices(&self) -> &[std::sync::Arc<VkPhysicalDevice>] {
        self.usable_physical_devices.as_slice()
    }

    pub fn unusable_physical_devices(&self) -> &[std::sync::Arc<VkUnusablePhysicalDevice>] {
        self.unusable_physical_devices.as_slice()
    }

    pub fn log_all(&self) {
        log::info!("List of all available physical devices:");
        if !self.usable_physical_devices().is_empty() {
            log::info!("Usable physical devices:");
        }
        for device in self.usable_physical_devices().iter() {
            log::info!(" - {device}");
        }
        if !self.unusable_physical_devices().is_empty() {
            log::info!("Unusable physical devices:");
        }
        for device in self.unusable_physical_devices().iter() {
            log::info!(" - {device} Unusubale: {}", device.reason);
        }
    }
}

pub fn query_physical_devices(
    instance: &ash::Instance,
    surface: ash::vk::SurfaceKHR,
    surface_instance: &ash::khr::surface::Instance,
) -> ash::prelude::VkResult<VkPhysicalDevices> {
    let physical_devices = unsafe { instance.enumerate_physical_devices() }?;

    let mut usable_physical_devices = Vec::with_capacity(physical_devices.len());
    let mut unusable_physical_devices = Vec::with_capacity(physical_devices.len());

    for physical_device in physical_devices.into_iter() {
        match check_physical_device_capabilities(instance, surface, surface_instance, physical_device)? {
            DeviceClassification::Usable(physical_device) => {
                let physical_device = std::sync::Arc::new(physical_device);
                usable_physical_devices.push(physical_device)
            }
            DeviceClassification::Unusable(physical_device) => {
                let physical_device = std::sync::Arc::new(physical_device);
                unusable_physical_devices.push(physical_device)
            }
        }
    }

    Ok(VkPhysicalDevices {
        usable_physical_devices,
        unusable_physical_devices,
    })
}

fn check_physical_device_capabilities(
    instance: &ash::Instance,
    surface: ash::vk::SurfaceKHR,
    surface_instance: &ash::khr::surface::Instance,
    physical_device: ash::vk::PhysicalDevice,
) -> ash::prelude::VkResult<DeviceClassification> {
    use vulkan::utils::vulkan_name_to_string;
    use vulkan::QueueFamilyIndices;

    /* Get all required properties to check */

    /* Use the p_next pointer chain to get 1.2 & 1.3 properties */
    let mut properties_1_3 = ash::vk::PhysicalDeviceVulkan13Properties::default();
    let mut properties_1_2 = ash::vk::PhysicalDeviceVulkan12Properties::default();
    let mut properties = ash::vk::PhysicalDeviceProperties2::default()
        .push_next(&mut properties_1_3)
        .push_next(&mut properties_1_2);
    unsafe { instance.get_physical_device_properties2(physical_device, &mut properties) };
    let properties = properties.properties;

    /* Straight forward for memory properties, no extensions to build */
    let mut memory_properties = ash::vk::PhysicalDeviceMemoryProperties2::default();
    unsafe { instance.get_physical_device_memory_properties2(physical_device, &mut memory_properties) };
    let memory_properties = memory_properties.memory_properties;

    /* Use the p_next pointer chain to get 1.2 & 1.3 features */
    let mut features_1_3 = ash::vk::PhysicalDeviceVulkan13Features::default();
    let mut features_1_2 = ash::vk::PhysicalDeviceVulkan12Features::default();
    let mut features = ash::vk::PhysicalDeviceFeatures2::default()
        .push_next(&mut features_1_3)
        .push_next(&mut features_1_2);
    unsafe { instance.get_physical_device_features2(physical_device, &mut features) };
    let features = features.features;

    let queue_families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let extensions = unsafe { instance.enumerate_device_extension_properties(physical_device) }?;
    let supported_extensions = extensions
        .into_iter()
        .map(|ext| vulkan_name_to_string(ext.extension_name.as_slice()))
        .collect::<std::collections::BTreeSet<_>>();

    let surface_support = SurfaceSupport::get(surface, surface_instance, physical_device)?;

    /* Build the names from the vulkan c strings */
    let name = vulkan_name_to_string(properties.device_name.as_slice());
    let driver = vulkan_name_to_string(properties_1_2.driver_name.as_slice());

    if properties.api_version < vulkan::VULKAN_API_VERSION {
        return Ok(DeviceClassification::Unusable(VkUnusablePhysicalDevice {
            name,
            driver,
            handle: physical_device,
            reason: format!(
                "Invalid API version: required {}, found {}",
                vulkan::utils::format_api_version(vulkan::VULKAN_API_VERSION),
                vulkan::utils::format_api_version(properties.api_version),
            ),
        }));
    }

    let queue_family_indices = match QueueFamilyIndices::get(instance, surface_instance, surface, physical_device)? {
        Some(queue_family) => queue_family,
        None => {
            return Ok(DeviceClassification::Unusable(VkUnusablePhysicalDevice {
                name,
                driver,
                handle: physical_device,
                reason: format!("No suitable graphics queue family"),
            }))
        }
    };

    /* Check the physical device has all the required features */
    let features_check = requirements::physical_device_features_check(features, features_1_2, features_1_3);
    if !features_check {
        return Ok(DeviceClassification::Unusable(VkUnusablePhysicalDevice {
            name,
            driver,
            handle: physical_device,
            reason: format!("Device does not have the required features"),
        }));
    }

    /* Check the physical device has all the required extensions */
    let extensions_check = requirements::physical_device_extensions_check(&supported_extensions);
    if !extensions_check {
        return Ok(DeviceClassification::Unusable(VkUnusablePhysicalDevice {
            name,
            driver,
            handle: physical_device,
            reason: format!("Device does not have the required extensions"),
        }));
    }

    /* check the surface can be supported by the device */
    if surface_support.formats().is_empty() || surface_support.present_modes().is_empty() {
        return Ok(DeviceClassification::Unusable(VkUnusablePhysicalDevice {
            name,
            driver,
            handle: physical_device,
            reason: format!("Device has insufficient surface support"),
        }));
    }

    Ok(DeviceClassification::Usable(VkPhysicalDevice {
        name,
        driver,
        handle: physical_device,
        properties,
        properties_1_2,
        properties_1_3,
        memory_properties,
        features,
        features_1_2,
        features_1_3,
        queue_families,
        supported_extensions,
        surface_support,
        queue_family_indices,
    }))
}

/// Container regrouping usable and unusable devices.
enum DeviceClassification {
    Usable(VkPhysicalDevice),
    Unusable(VkUnusablePhysicalDevice),
}
