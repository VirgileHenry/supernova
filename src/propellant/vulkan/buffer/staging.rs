use crate::propellant::vulkan;
use crate::propellant::vulkan::buffer::*;

/// Marker struct for staging buffers.
pub struct Staging {
    /// Persistent mapping into the staging buffer's memory.
    mapped: std::ptr::NonNull<u8>,
    /// Size of the mapping in bytes.
    size: u64,
}

impl BufferUsage for Staging {
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
        ash::vk::BufferUsageFlags::TRANSFER_SRC
    }
    fn memory_property_flags() -> ash::vk::MemoryPropertyFlags {
        ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT
    }
}

impl TransferSrc for Staging {}

impl<T: Copy> VkBuffer<Staging, T> {
    /// Write data into the staging buffer at the given byte offset.
    ///
    /// Returns whether the write happened or not.
    /// If the function returns false, it's likely the provided data
    /// could not fit in the buffer at the given offset.
    pub fn write(&mut self, offset: usize, data: &[T]) -> Result<BufferView<T>, BufferWriteError> {
        if offset + data.len() <= self.capacity.get() {
            let bytes_to_write = data.len() * std::mem::size_of::<T>();
            let byte_offset = offset * std::mem::size_of::<T>();

            unsafe {
                std::ptr::copy_nonoverlapping(
                    data.as_ptr() as *const u8,
                    self.usage.mapped.as_ptr().add(byte_offset),
                    bytes_to_write,
                );
            }

            Ok(BufferView {
                offset,
                length: data.len(),
                _m: std::marker::PhantomData,
            })
        } else {
            Err(BufferWriteError::new(self.capacity.get(), offset, data.len()))
        }
    }
}
