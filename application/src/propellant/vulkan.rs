mod buffer;
mod command_interface;
mod device;
mod error;
mod instance;
mod physical_devices;
mod queue;
mod queue_family_indices;
mod reference;
mod swapchain;
mod synchronization;
mod utils;

pub use buffer::copy_buffer;
pub use buffer::BufferView;
pub use buffer::RingInstanceBuffer;
pub use buffer::RingStorageBuffer;
pub use buffer::RingUniformBuffer;
pub use buffer::StagingBuffer;
pub use buffer::StorageBuffer;
pub use buffer::UniformBuffer;
pub use command_interface::CommandInterface;
pub use command_interface::InRecordingCommandBuffer;
pub use device::VkDeviceHandle;
pub use error::VkError;
pub use physical_devices::VkPhysicalDevice;
pub use physical_devices::VkPhysicalDevices;
pub use queue::VkQueue;
pub use queue::VkSharableQueue;
pub use queue_family_indices::QueueFamilyIndices;
pub use reference::VkRef;
pub use swapchain::VkSwapchain;
pub use synchronization::VkSynchronization;

pub const VULKAN_API_VERSION: u32 = ash::vk::make_api_version(0, 1, 3, 0);
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct VulkanContext {
    vk_device: std::sync::Arc<device::VkDevice>,
    vk_instance: instance::VkInstance,
    window: winit::window::Window,
}

impl VulkanContext {
    pub fn create(window: winit::window::Window) -> Result<Self, VkError> {
        let vk_instance = instance::VkInstance::create(&window)?;
        let prefered_physical_device = vk_instance
            .prefered_physical_device()
            .ok_or_else(|| VkError::MissingPhysicalDevice {
                device_names: vk_instance.physical_device_names().map(String::from).collect(),
            })?;

        let vk_device = device::VkDevice::create(&vk_instance, prefered_physical_device.clone())?;

        Ok(Self {
            vk_device,
            vk_instance,
            window,
        })
    }

    pub fn device_handle(&self) -> VkDeviceHandle {
        self.vk_device.clone()
    }

    pub fn instance(&self) -> &instance::VkInstance {
        &self.vk_instance
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}
