use super::utils::VkShared;
use ash::vk;

/// Extensions that are required to run supernova
/// This shall match vulka::physical_deice::REQUIRED_DEVICE_EXTENSIONS
const REQUIRED_DEVICE_EXTENSIONS: &[*const i8] = &[
    ash::vk::KHR_SWAPCHAIN_NAME.as_ptr(),
    ash::vk::EXT_DESCRIPTOR_INDEXING_NAME.as_ptr(),
];

#[derive(Debug, Clone, Copy)]
pub struct QueueInterface {
    pub queue: ash::vk::Queue,
    pub family: u32,
}

impl std::ops::Deref for QueueInterface {
    type Target = ash::vk::Queue;
    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}

// Should this be implemented as a trait, avoid to use before thinking about the best way to implement this
pub trait VulkanPhysicalProperties {
    fn get_physical_device_properties(&self) -> &ash::vk::PhysicalDeviceProperties;
    fn get_physical_device_memory_properties(&self) -> &ash::vk::PhysicalDeviceMemoryProperties;
}

/// Structure that contains all the vulkan stuff used and needed by the renderer.
/// This is owned by the renderer, as it is used every frame.
pub struct VulkanInterface {
    device: ash::Device,
    physical_device: ash::vk::PhysicalDevice,
    physical_device_properties: ash::vk::PhysicalDeviceProperties,
    physical_device_memory_properties: ash::vk::PhysicalDeviceMemoryProperties,

    graphics_queue: QueueInterface,
    present_queue: OwnOrSharedQueue,
    transfer_queue: OwnOrSharedQueue,
    compute_queue: OwnOrSharedQueue,
}

impl VulkanInterface {
    pub fn create(
        vk_instance: &ash::Instance,
        physical_device: ash::vk::PhysicalDevice,
        indices: &crate::propellant::vulkan::QueueFamilyIndices,
    ) -> Result<VulkanInterface, crate::ScError> {
        // at most, we will have 4 queues from the same queue family, so we'll need a list of at most 4 elements
        let queue_priorities = [1.0; 4];

        let graphic_queue_info = ash::vk::DeviceQueueCreateInfo {
            queue_count: 1,
            queue_family_index: indices.graphics.0,
            p_queue_priorities: queue_priorities.as_ptr(),
            ..Default::default()
        };

        let mut queue_create_infos = std::collections::HashMap::new();
        queue_create_infos.insert(indices.graphics.0, graphic_queue_info);

        if let Some((present_index, _)) = indices.present {
            match queue_create_infos.get_mut(&present_index) {
                Some(create_info) => create_info.queue_count += 1,
                None => {
                    queue_create_infos.insert(
                        present_index,
                        ash::vk::DeviceQueueCreateInfo {
                            queue_count: 1,
                            queue_family_index: present_index,
                            p_queue_priorities: queue_priorities.as_ptr(),
                            ..Default::default()
                        },
                    );
                }
            }
        }

        if let Some((transfer_index, _)) = indices.transfer {
            match queue_create_infos.get_mut(&transfer_index) {
                Some(create_info) => create_info.queue_count += 1,
                None => {
                    queue_create_infos.insert(
                        transfer_index,
                        ash::vk::DeviceQueueCreateInfo {
                            queue_count: 1,
                            queue_family_index: transfer_index,
                            p_queue_priorities: queue_priorities.as_ptr(),
                            ..Default::default()
                        },
                    );
                }
            }
        }

        if let Some((compute_index, _)) = indices.compute {
            match queue_create_infos.get_mut(&compute_index) {
                Some(create_info) => create_info.queue_count += 1,
                None => {
                    queue_create_infos.insert(
                        compute_index,
                        ash::vk::DeviceQueueCreateInfo {
                            queue_count: 1,
                            queue_family_index: compute_index,
                            p_queue_priorities: queue_priorities.as_ptr(),
                            ..Default::default()
                        },
                    );
                }
            }
        }

        let queue_create_infos = queue_create_infos.into_values().collect::<Vec<_>>();

        // TODO: we already have a required device feature ? See crate::vulkan::physical_devices
        let features = ash::vk::PhysicalDeviceFeatures { ..Default::default() };

        for extension in REQUIRED_DEVICE_EXTENSIONS.iter() {
            if extension.is_null() {
                log::warn!("Device extension name pointer is null!!");
            } else {
                log::info!("Using device extension {:#?}", unsafe {
                    std::ffi::CStr::from_ptr(*extension)
                });
            }
        }

        let info = ash::vk::DeviceCreateInfo {
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: if queue_create_infos.is_empty() {
                std::ptr::null()
            } else {
                queue_create_infos.as_ptr()
            },
            enabled_extension_count: REQUIRED_DEVICE_EXTENSIONS.len() as u32,
            pp_enabled_extension_names: if REQUIRED_DEVICE_EXTENSIONS.is_empty() {
                std::ptr::null()
            } else {
                REQUIRED_DEVICE_EXTENSIONS.as_ptr()
            },
            p_enabled_features: &features,
            ..Default::default()
        };

        let device = unsafe { vk_instance.create_device(physical_device, &info, None)? };

        let graphics_queue = QueueInterface {
            queue: unsafe { device.get_device_queue(indices.graphics.0, indices.graphics.1) },
            family: indices.graphics.0,
        };

        let present_queue = match indices.present {
            Some((indices, offset)) => OwnOrSharedQueue::Owned(QueueInterface {
                queue: unsafe { device.get_device_queue(indices, offset) },
                family: indices,
            }),
            None => OwnOrSharedQueue::Shared(VkShared::shared(&graphics_queue)),
        };
        let transfer_queue = match indices.transfer {
            Some((indices, offset)) => OwnOrSharedQueue::Owned(QueueInterface {
                queue: unsafe { device.get_device_queue(indices, offset) },
                family: indices,
            }),
            None => OwnOrSharedQueue::Shared(VkShared::shared(&graphics_queue)),
        };
        let compute_queue = match indices.compute {
            Some((indices, offset)) => OwnOrSharedQueue::Owned(QueueInterface {
                queue: unsafe { device.get_device_queue(indices, offset) },
                family: indices,
            }),
            None => OwnOrSharedQueue::Shared(VkShared::shared(&graphics_queue)),
        };

        log::info!("Created logical device with the following queues:");
        log::info!(" ├─ Graphics: Owned");
        log::info!(" ├─ Present: {:?}", present_queue.ownership());
        log::info!(" ├─ Transfer: {:?}", transfer_queue.ownership());
        log::info!(" └─ Compute: {:?}", compute_queue.ownership());

        let physical_device_properties = unsafe { vk_instance.get_physical_device_properties(physical_device) };
        let physical_device_memory_properties = unsafe { vk_instance.get_physical_device_memory_properties(physical_device) };

        Ok(VulkanInterface {
            device,
            physical_device,
            physical_device_properties,
            physical_device_memory_properties,
            graphics_queue,
            present_queue,
            transfer_queue,
            compute_queue,
        })
    }

    pub fn get_device(&self) -> &ash::Device {
        &self.device
    }

    pub fn get_physical_device(&self) -> &ash::vk::PhysicalDevice {
        &self.physical_device
    }

    pub fn renderer_interface(&self) -> VkRendererInterface {
        VkRendererInterface {
            device: VkShared::shared(&self.device),
            graphics_queue: VkShared::shared(&self.graphics_queue),
            present_queue: match &self.present_queue {
                OwnOrSharedQueue::Owned(queue) => VkShared::shared(queue),
                OwnOrSharedQueue::Shared(shared) => shared.clone(),
            },
            physical_device_properties: self.physical_device_properties,
            physical_device_memory_properties: self.physical_device_memory_properties,
        }
    }

    pub fn asset_manager_interface(&self) -> VkAssetManagerInterface {
        VkAssetManagerInterface {
            device: VkShared::shared(&self.device),
            transfer_queue: match &self.transfer_queue {
                OwnOrSharedQueue::Owned(queue) => VkShared::shared(queue),
                OwnOrSharedQueue::Shared(shared) => shared.clone(),
            },
            physical_device_properties: self.physical_device_properties,
            physical_device_memory_properties: self.physical_device_memory_properties,
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            match self.device.device_wait_idle() {
                Ok(_) => {}
                Err(e) => log::warn!("Failed to wait device idle for cleanup: {e}"),
            }
            self.device.destroy_device(None);
        }
    }
}

impl VulkanPhysicalProperties for VulkanInterface {
    fn get_physical_device_properties(&self) -> &ash::vk::PhysicalDeviceProperties {
        &self.physical_device_properties
    }

    fn get_physical_device_memory_properties(&self) -> &ash::vk::PhysicalDeviceMemoryProperties {
        &self.physical_device_memory_properties
    }
}

enum OwnOrSharedQueue {
    Owned(QueueInterface),
    Shared(VkShared<QueueInterface>),
}

impl OwnOrSharedQueue {
    fn ownership(&self) -> &'static str {
        match self {
            OwnOrSharedQueue::Owned(_) => "Owned",
            OwnOrSharedQueue::Shared(_) => "Shared",
        }
    }
}

pub struct VkRendererInterface {
    device: VkShared<ash::Device>,
    graphics_queue: VkShared<QueueInterface>,
    present_queue: VkShared<QueueInterface>,
    physical_device_properties: vk::PhysicalDeviceProperties,
    physical_device_memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl std::ops::Deref for VkRendererInterface {
    type Target = ash::Device;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl VulkanPhysicalProperties for VkRendererInterface {
    fn get_physical_device_properties(&self) -> &ash::vk::PhysicalDeviceProperties {
        &self.physical_device_properties
    }

    fn get_physical_device_memory_properties(&self) -> &ash::vk::PhysicalDeviceMemoryProperties {
        &self.physical_device_memory_properties
    }
}

impl VkRendererInterface {
    pub fn load_shader(&self, byte_code: &[u32]) -> Result<ash::vk::ShaderModule, crate::ScError> {
        let create_info = ash::vk::ShaderModuleCreateInfo {
            code_size: byte_code.len() * std::mem::size_of::<u32>(),
            p_code: if byte_code.is_empty() {
                std::ptr::null()
            } else {
                byte_code.as_ptr()
            },
            ..Default::default()
        };
        Ok(unsafe { self.device.create_shader_module(&create_info, None)? })
    }

    pub fn graphics_queue(&self) -> &QueueInterface {
        &self.graphics_queue
    }

    pub fn present_queue(&self) -> &QueueInterface {
        &self.present_queue
    }
}

pub struct VkAssetManagerInterface {
    pub device: VkShared<ash::Device>,
    pub transfer_queue: VkShared<QueueInterface>,
    physical_device_properties: vk::PhysicalDeviceProperties,
    physical_device_memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl std::ops::Deref for VkAssetManagerInterface {
    type Target = ash::Device;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl VulkanPhysicalProperties for VkAssetManagerInterface {
    fn get_physical_device_properties(&self) -> &ash::vk::PhysicalDeviceProperties {
        &self.physical_device_properties
    }

    fn get_physical_device_memory_properties(&self) -> &ash::vk::PhysicalDeviceMemoryProperties {
        &self.physical_device_memory_properties
    }
}

pub struct CommandInterface {
    queue: VkShared<QueueInterface>,
    command_pool: ash::vk::CommandPool,
    command_buffers: Vec<ash::vk::CommandBuffer>,
}

impl CommandInterface {
    pub fn create(device: &ash::Device, queue: &QueueInterface, frame_count: u32) -> Result<CommandInterface, crate::ScError> {
        let queue = VkShared::shared(queue);
        let create_info = ash::vk::CommandPoolCreateInfo {
            queue_family_index: queue.family,
            flags: ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            ..Default::default()
        };
        let command_pool = unsafe { device.create_command_pool(&create_info, None)? };
        let command_buffers = Self::create_command_buffers(device, command_pool, frame_count)?;
        Ok(CommandInterface {
            queue,
            command_pool,
            command_buffers,
        })
    }

    pub fn recreate_command_buffers(&mut self, device: &ash::Device, frame_count: u32) -> Result<(), crate::ScError> {
        self.command_buffers = Self::create_command_buffers(device, self.command_pool, frame_count)?;
        Ok(())
    }

    pub fn queue(&self) -> ash::vk::Queue {
        self.queue.queue
    }

    pub fn start_recording(&self, device: &ash::Device, index: usize) -> InRecordingCommandBuffer {
        let command_buffer = self.command_buffers[index];
        unsafe {
            if let Err(e) = device.reset_command_buffer(command_buffer, ash::vk::CommandBufferResetFlags::empty()) {
                log::warn!("Error when resetting command bufffer: {e}");
            }
            if let Err(e) = device.begin_command_buffer(command_buffer, &ash::vk::CommandBufferBeginInfo::default()) {
                log::warn!("Error when begining comand buffer recording: {e}");
            }
        }
        InRecordingCommandBuffer(VkShared::shared(&command_buffer))
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_command_pool(self.command_pool, None);
        }
    }

    fn create_command_buffers(
        device: &ash::Device,
        command_pool: ash::vk::CommandPool,
        frame_count: u32,
    ) -> Result<Vec<ash::vk::CommandBuffer>, crate::ScError> {
        let allocate_info = ash::vk::CommandBufferAllocateInfo {
            command_pool,
            level: ash::vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: frame_count,
            ..Default::default()
        };
        Ok(unsafe { device.allocate_command_buffers(&allocate_info)? })
    }
}

pub struct InRecordingCommandBuffer(VkShared<ash::vk::CommandBuffer>);

impl std::ops::Deref for InRecordingCommandBuffer {
    type Target = ash::vk::CommandBuffer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl InRecordingCommandBuffer {
    pub fn end_recording(self, device: &ash::Device) -> RecordedCommandBuffer {
        let InRecordingCommandBuffer(command_buffer) = self;
        if let Err(e) = unsafe { device.end_command_buffer(*command_buffer) } {
            log::warn!("Error when ending command buffer recording: {e}");
        }
        RecordedCommandBuffer(command_buffer)
    }
}

pub struct RecordedCommandBuffer(VkShared<ash::vk::CommandBuffer>);

impl std::ops::Deref for RecordedCommandBuffer {
    type Target = ash::vk::CommandBuffer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RecordedCommandBuffer {
    pub fn queue_submit(
        self,
        device: &ash::Device,
        queue: ash::vk::Queue,
        submit_infos: &[ash::vk::SubmitInfo],
        fence: ash::vk::Fence,
    ) -> Result<(), crate::ScError> {
        unsafe {
            device.queue_submit(queue, submit_infos, fence)?;
        }
        Ok(())
    }
}
