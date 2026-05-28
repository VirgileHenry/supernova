use crate::propellant::vulkan;
use crate::propellant::vulkan::buffer::*;

/// Marker struct for uniform buffers.
pub struct Uniform {
    /// Persistent mapping into the uniform buffer's memory.
    mapped: std::ptr::NonNull<u8>,
    /// Size of the mapping in bytes.
    size: u64,
}

impl BufferUsage for Uniform {
    fn create(device: &vulkan::VkDeviceHandle, memory: ash::vk::DeviceMemory, size: u64) -> ash::prelude::VkResult<Self> {
        let ptr = unsafe { device.map_memory(memory, 0, size, ash::vk::MemoryMapFlags::empty()) }?;
        // SAFETY: map_memory returns non-null on success.
        let mapped = unsafe { std::ptr::NonNull::new_unchecked(ptr as *mut u8) };
        Ok(Self { mapped, size })
    }
    fn destroy(&mut self, device: &vulkan::VkDeviceHandle, memory: ash::vk::DeviceMemory) {
        unsafe { device.unmap_memory(memory) };
    }
    fn buffer_usage_flags() -> ash::vk::BufferUsageFlags {
        ash::vk::BufferUsageFlags::UNIFORM_BUFFER
    }
    fn memory_property_flags() -> ash::vk::MemoryPropertyFlags {
        ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT
    }
}

impl TransferSrc for Uniform {}

impl<T: Copy> VkBuffer<Uniform, T> {
    /// Write data into the staging buffer at the given byte offset.
    ///
    /// Returns whether the write happened or not.
    /// If the function returns false, it's likely the provided data
    /// could not fit in the buffer at the given offset.
    pub fn write(&mut self, data: &T) -> BufferView<T> {
        unsafe {
            let bytes_to_write = std::mem::size_of::<T>();
            std::ptr::copy_nonoverlapping(data as *const T as *const u8, self.usage.mapped.as_ptr(), bytes_to_write);
        }

        BufferView {
            offset: 0,
            length: 1,
            _m: std::marker::PhantomData,
        }
    }
}
