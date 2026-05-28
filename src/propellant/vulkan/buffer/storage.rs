use crate::propellant::vulkan;
use crate::propellant::vulkan::buffer::*;

/// Marker struct for storage buffers.
pub struct Storage;

impl BufferUsage for Storage {
    fn create(_: &vulkan::VkDeviceHandle, _: ash::vk::DeviceMemory, _: u64) -> ash::prelude::VkResult<Self> {
        Ok(Self)
    }
    fn destroy(&mut self, _: &vulkan::VkDeviceHandle, _: ash::vk::DeviceMemory) {}
    fn buffer_usage_flags() -> ash::vk::BufferUsageFlags {
        ash::vk::BufferUsageFlags::STORAGE_BUFFER | ash::vk::BufferUsageFlags::TRANSFER_DST
    }
    fn memory_property_flags() -> ash::vk::MemoryPropertyFlags {
        ash::vk::MemoryPropertyFlags::DEVICE_LOCAL
    }
}

impl TransferDst for Storage {}
