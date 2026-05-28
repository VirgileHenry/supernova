use crate::propellant::vulkan;

/// Structure to hold a per frame uniform buffer.
pub struct UniformBuffer<T> {
    /// One per image in flight
    buffers: Vec<vulkan::UniformBuffer<T>>,
}

impl<T> UniformBuffer<T> {
    pub fn create(vk_device: &vulkan::VkDeviceHandle) -> ash::prelude::VkResult<Self> {
        let buffers: Vec<_> = (0..vulkan::MAX_FRAMES_IN_FLIGHTS)
            .map(|_| vulkan::UniformBuffer::create(vk_device, std::num::NonZeroUsize::new(1).unwrap()))
            .collect::<Result<_, _>>()?;

        Ok(Self { buffers })
    }
}

impl<T: Copy> UniformBuffer<T> {
    pub fn write(&mut self, value: &T, frame_index: usize) {
        self.buffers[frame_index].write(value);
    }
}
