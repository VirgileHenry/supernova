mod error;
mod staging;
mod storage;
mod uniform;
mod utils;

use crate::propellant::vulkan;
use error::BufferWriteError;

/// Storage buffer type shortcut.
pub type StorageBuffer<T> = VkBuffer<storage::Storage, T>;

/// Staging buffer type shortcut.
pub type StagingBuffer<T> = VkBuffer<staging::Staging, T>;

/// Uniform buffer type shortcut.
pub type UniformBuffer<T> = VkBuffer<uniform::Uniform, T>;

/// Marker trait for different kind of buffers.
pub trait BufferUsage: Sized {
    /// Called after the buffer is created and bound to memory.
    /// May map the memory and store the pointer if needed.
    fn create(vk_device: &vulkan::VkDeviceHandle, memory: ash::vk::DeviceMemory, size: u64) -> ash::prelude::VkResult<Self>;

    /// Called when the buffer is dropped, before memory/buffer destruction.
    /// Implementations should unmap memory if they mapped it.
    fn destroy(&mut self, device: &vulkan::VkDeviceHandle, memory: ash::vk::DeviceMemory);

    fn buffer_usage_flags() -> ash::vk::BufferUsageFlags;
    fn memory_property_flags() -> ash::vk::MemoryPropertyFlags;
}

/// Marker trait for buffers that can be used as transfer destinations
pub trait TransferDst: BufferUsage {}

/// Marker trait for buffers that can be used as transfer destinations
pub trait TransferSrc: BufferUsage {}

/// Wrapper around a Vulkan buffer.
///
/// Ths buffer usage is provided as a generic parameter,
/// allowing type safety on buffer usages.
pub struct VkBuffer<Usage: BufferUsage, T> {
    /// Vulkan device handle to perform GPU operations on our own.
    vk_device: vulkan::VkDeviceHandle,

    /// Buffer handle.
    handle: ash::vk::Buffer,
    /// Backing device memory.
    memory: ash::vk::DeviceMemory,

    /// Size of the buffer in bytes.
    capacity: std::num::NonZeroUsize,

    /// Buffer usage, in case the usage struct carries any data, like mapped memory.
    usage: Usage,

    /// Type marker for the inner stored type T.
    _m: std::marker::PhantomData<T>,
}

impl<Usage: BufferUsage, T> VkBuffer<Usage, T> {
    pub fn create(vk_device: &vulkan::VkDeviceHandle, capacity: std::num::NonZeroUsize) -> ash::prelude::VkResult<Self> {
        let buffer_size: usize = std::mem::size_of::<T>() * capacity.get();
        let buffer_size: u64 = buffer_size as u64;

        let buffer_info = ash::vk::BufferCreateInfo::default()
            .size(buffer_size)
            .usage(Usage::buffer_usage_flags())
            .sharing_mode(ash::vk::SharingMode::EXCLUSIVE);

        let handle = unsafe { vk_device.create_buffer(&buffer_info, None) }?;

        let mem_requirements = unsafe { vk_device.get_buffer_memory_requirements(handle) };
        let memory_properties = Usage::memory_property_flags();

        let memory_type_index = match utils::find_memory_type(
            vk_device.physical_device(),
            mem_requirements.memory_type_bits,
            memory_properties,
        ) {
            Some(idx) => idx,
            None => {
                unsafe { vk_device.destroy_buffer(handle, None) };
                return Err(ash::vk::Result::ERROR_FEATURE_NOT_PRESENT);
            }
        };

        let alloc_info = ash::vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index);

        let memory = match unsafe { vk_device.allocate_memory(&alloc_info, None) } {
            Ok(memory) => memory,
            Err(e) => {
                unsafe { vk_device.destroy_buffer(handle, None) };
                return Err(e);
            }
        };

        if let Err(e) = unsafe { vk_device.bind_buffer_memory(handle, memory, 0) } {
            unsafe {
                vk_device.free_memory(memory, None);
                vk_device.destroy_buffer(handle, None);
            }
            return Err(e);
        }

        let usage = Usage::create(vk_device, memory, mem_requirements.size)?;

        Ok(Self {
            vk_device: vk_device.clone(),
            handle,
            memory,
            capacity,
            usage,
            _m: std::marker::PhantomData,
        })
    }

    pub fn capacity(&self) -> usize {
        self.capacity.get()
    }

    pub fn size(&self) -> usize {
        self.capacity() * std::mem::size_of::<T>()
    }

    pub fn entire_buffer(&self) -> BufferView<T> {
        BufferView {
            offset: 0,
            length: self.capacity.get(),
            _m: std::marker::PhantomData,
        }
    }
}

impl<Usage: BufferUsage, T> Drop for VkBuffer<Usage, T> {
    fn drop(&mut self) {
        self.usage.destroy(&self.vk_device, self.memory);
        unsafe { self.vk_device.destroy_buffer(self.handle, None) };
        unsafe { self.vk_device.free_memory(self.memory, None) };
    }
}

pub fn copy_buffer<Src: TransferSrc, Dst: TransferDst, T>(
    vk_device: &vulkan::VkDeviceHandle,
    source: &VkBuffer<Src, T>,
    destination: &mut VkBuffer<Dst, T>,
    source_view: BufferView<T>,
    destination_view: BufferView<T>,
) -> ash::prelude::VkResult<()> {
    if source_view.length != destination_view.length {
        log::warn!("Unable to copy buffers: source and destination size mismatch");
        return Ok(()); /* Fixme: error  */
    }
    /* This is clanker code, clean up. */

    // Get the transfer queue and its family.
    let transfer_queue = vk_device.transfer_queue();

    /* Fixme: let's not allocate a pool at each buffer copy */
    // Create a transient command pool for this one-shot operation.
    let pool_info = ash::vk::CommandPoolCreateInfo::default()
        .queue_family_index(transfer_queue.family())
        .flags(ash::vk::CommandPoolCreateFlags::TRANSIENT);
    let command_pool = unsafe { vk_device.create_command_pool(&pool_info, None) }?;

    // Allocate one command buffer.
    let alloc_info = ash::vk::CommandBufferAllocateInfo::default()
        .command_pool(command_pool)
        .level(ash::vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    let command_buffers = match unsafe { vk_device.allocate_command_buffers(&alloc_info) } {
        Ok(bufs) => bufs,
        Err(e) => {
            unsafe { vk_device.destroy_command_pool(command_pool, None) };
            return Err(e);
        }
    };
    let cmd = command_buffers[0];

    // Record the copy.
    let begin_info = ash::vk::CommandBufferBeginInfo::default().flags(ash::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { vk_device.begin_command_buffer(cmd, &begin_info) }?;

    let region = ash::vk::BufferCopy::default()
        .src_offset((source_view.offset * std::mem::size_of::<T>()) as u64)
        .dst_offset((destination_view.offset * std::mem::size_of::<T>()) as u64)
        .size((source_view.length * std::mem::size_of::<T>()) as u64);

    unsafe { vk_device.cmd_copy_buffer(cmd, source.handle, destination.handle, &[region]) };
    unsafe { vk_device.end_command_buffer(cmd) }?;

    // Submit, with a fence so we can wait.
    let fence_info = ash::vk::FenceCreateInfo::default();
    let fence = unsafe { vk_device.create_fence(&fence_info, None) }?;

    let cmd_buffers = [cmd];
    let submit = ash::vk::SubmitInfo::default().command_buffers(&cmd_buffers);
    let result = unsafe { vk_device.queue_submit(**transfer_queue, &[submit], fence) };

    if result.is_ok() {
        unsafe { vk_device.wait_for_fences(&[fence], true, u64::MAX) }?;
    }

    // Clean up.
    unsafe {
        vk_device.destroy_fence(fence, None);
        vk_device.destroy_command_pool(command_pool, None); // frees command buffers
    }

    result
}

/// View into a given buffer.
///
/// This is a offset + length pair that targets a memory zone into a GPU buffer.
///
/// Both offset and length are counted in the number of Ts, not bytes.
#[derive(Debug, Clone, Copy)]
pub struct BufferView<T> {
    offset: usize,
    length: usize,
    _m: std::marker::PhantomData<T>,
}

impl<T> BufferView<T> {
    pub fn empty() -> Self {
        Self {
            offset: 0,
            length: 0,
            _m: std::marker::PhantomData,
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn length(&self) -> usize {
        self.length
    }
}
