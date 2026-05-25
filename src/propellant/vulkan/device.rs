use crate::propellant::vulkan;

/// Shared handle to the Vulkan device.
///
/// This type allow each subsystem to access the vulkan device and interface.
pub type VkDeviceHandle = std::sync::Arc<VkDevice>;

/// Abstraction over the Vulkan device.
///
/// This contains the Vulkan device handle, but also the physical device
/// it was built on, as well as the different queues that can be used.
#[derive(Clone)]
pub struct VkDevice {
    /// Reference to the used physical device this device is based on.
    physical_device: std::sync::Arc<vulkan::VkPhysicalDevice>,

    /// Vulkan device.
    device: ash::Device,

    /// Graphics queue.
    /// It has to be owned, since we need to have at least a graphics queue.
    graphics_queue: vulkan::VkQueue,
    /// Transfer queue.
    /// Either shared with another queue, or a dedicated transfer queue.
    transfer_queue: vulkan::VkSharableQueue,
    /// Compute queue.
    /// Either shared with another queue, or a dedicated compute queue.
    compute_queue: vulkan::VkSharableQueue,
}

impl VkDevice {
    pub fn create(
        vk_instance: &ash::Instance,
        physical_device: std::sync::Arc<vulkan::VkPhysicalDevice>,
    ) -> ash::prelude::VkResult<std::sync::Arc<Self>> {
        let required_extensions = vulkan::physical_devices::required_device_extensions();
        let required_features = vulkan::physical_devices::required_device_features();

        let queue_family_indices = physical_device.queue_family_indices();
        let queue_create_infos = queue_family_indices.to_queue_create_infos();

        let required_extension_names: Vec<_> = required_extensions
            .iter()
            .map(|extension| {
                log::info!("Using device extension {extension:#?}");
                extension.as_ptr() as *const i8
            })
            .collect();

        let device_create_info = ash::vk::DeviceCreateInfo::default()
            .queue_create_infos(queue_create_infos.as_slice())
            .enabled_extension_names(required_extension_names.as_slice())
            .enabled_features(&required_features);

        let device = unsafe { vk_instance.create_device(physical_device.handle(), &device_create_info, None)? };

        use vulkan::VkQueue;
        use vulkan::VkRef;
        use vulkan::VkSharableQueue;

        /* Keep track on queues that are already created to share them */
        let mut created_queues = std::collections::HashMap::new();

        /* Create the graphics queues in all cases, and track it in the created queues */
        let graphics_queue = VkQueue::create(&device, queue_family_indices.graphics);
        created_queues.insert(queue_family_indices.graphics, graphics_queue);

        /* If we have a dedicated transfer queue (no queue created from that family yet), create it. Otherwise, share */
        let transfer_queue = match created_queues.get(&queue_family_indices.transfer) {
            Some(queue) => VkSharableQueue::Shared(VkRef::shared(queue)),
            None => {
                let queue = VkQueue::create(&device, queue_family_indices.transfer);
                created_queues.insert(queue_family_indices.transfer, queue);
                VkSharableQueue::Owned(queue)
            }
        };
        /* If we have a dedicated compute queue (no queue created from that family yet), create it. Otherwise, share */
        let compute_queue = match created_queues.get(&queue_family_indices.compute) {
            Some(queue) => VkSharableQueue::Shared(VkRef::shared(queue)),
            None => {
                let queue = VkQueue::create(&device, queue_family_indices.compute);
                created_queues.insert(queue_family_indices.compute, queue);
                VkSharableQueue::Owned(queue)
            }
        };

        log::info!("Created logical device with the following queues:");
        log::info!(" ├─ Graphics: Owned");
        log::info!(" ├─ Transfer: {}", transfer_queue.ownership());
        log::info!(" └─ Compute: {}", compute_queue.ownership());

        Ok(std::sync::Arc::new(Self {
            device: device,
            physical_device,
            graphics_queue,
            transfer_queue,
            compute_queue,
        }))
    }

    /// Load a shader module
    pub fn load_shader_module(&self, byte_code: &[u32]) -> ash::prelude::VkResult<ash::vk::ShaderModule> {
        let create_info = ash::vk::ShaderModuleCreateInfo::default().code(byte_code);
        Ok(unsafe { self.device.create_shader_module(&create_info, None)? })
    }

    /// Get the used queue family indices of the physical device.
    pub fn physical_device(&self) -> &vulkan::VkPhysicalDevice {
        &self.physical_device
    }

    /// Get a shared reference to the graphics queue.
    pub fn graphics_queue(&self) -> vulkan::VkRef<vulkan::VkQueue> {
        vulkan::VkRef::shared(&self.graphics_queue)
    }

    /// Get a shared reference to the transfer queue.
    pub fn transfer_queue(&self) -> vulkan::VkRef<vulkan::VkQueue> {
        vulkan::VkRef::shared(self.transfer_queue.as_ref())
    }

    /// Get a shared reference to the compute queue.
    pub fn compute_queue(&self) -> vulkan::VkRef<vulkan::VkQueue> {
        vulkan::VkRef::shared(self.compute_queue.as_ref())
    }
}

impl std::ops::Deref for VkDevice {
    type Target = ash::Device;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl std::fmt::Debug for VkDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vk Device from {:?}", self.physical_device)
    }
}

impl Drop for VkDevice {
    fn drop(&mut self) {
        if let Err(e) = unsafe { self.device.device_wait_idle() } {
            log::warn!("Failed to wait device idle for cleanup: {e}");
        }
        unsafe { self.device.destroy_device(None) };
    }
}
