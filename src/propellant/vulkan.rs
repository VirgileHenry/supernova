mod command_interface;
mod device;
mod instance;
mod physical_devices;
mod queue;
mod queue_family_indices;
mod reference;
mod swapchain;
mod synchronization;
mod utils;

pub use command_interface::CommandInterface;
pub use command_interface::InRecordingCommandBuffer;
pub use device::VkDevice;
pub use device::VkDeviceHandle;
pub use instance::VkInstance;
pub use physical_devices::VkPhysicalDevice;
pub use physical_devices::VkPhysicalDevices;
pub use queue::VkQueue;
pub use queue::VkSharableQueue;
pub use queue_family_indices::QueueFamilyIndices;
pub use reference::VkRef;
pub use swapchain::VkSwapchain;
pub use synchronization::VkSynchronization;

pub const VULKAN_API_VERSION: u32 = ash::vk::make_api_version(0, 1, 3, 0);
pub const MAX_FRAMES_IN_FLIGHTS: usize = 2;
