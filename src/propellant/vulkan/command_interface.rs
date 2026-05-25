use ash::vk::VideoBeginCodingFlagsKHR;

use crate::propellant::vulkan;

pub struct CommandInterface {
    /// Handle to the vulkan device to use the vulkan API.
    vk_device: vulkan::VkDeviceHandle,

    /// Queue on which to submit command buffer work.
    queue: vulkan::VkRef<vulkan::VkQueue>,

    /// Command buffer pool to reuse them
    command_pool: ash::vk::CommandPool,
    /// In use command buffers
    command_buffers: Vec<ash::vk::CommandBuffer>,
}

impl CommandInterface {
    pub fn create(
        vk_device: &vulkan::VkDeviceHandle,
        queue: vulkan::VkRef<vulkan::VkQueue>,
        frame_count: u32,
    ) -> Result<CommandInterface, crate::ScError> {
        let create_info = ash::vk::CommandPoolCreateInfo {
            queue_family_index: queue.family(),
            flags: ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            ..Default::default()
        };
        let command_pool = unsafe { vk_device.create_command_pool(&create_info, None)? };
        let command_buffers = Self::create_command_buffers(vk_device, command_pool, frame_count)?;

        Ok(CommandInterface {
            vk_device: vk_device.clone(),
            queue,
            command_pool,
            command_buffers,
        })
    }

    pub fn recreate_command_buffers(&mut self, frame_count: u32) -> Result<(), crate::ScError> {
        let reset_flag = ash::vk::CommandPoolResetFlags::empty();
        unsafe { self.vk_device.reset_command_pool(self.command_pool, reset_flag) }?;
        if self.command_buffers.len() != frame_count as usize {
            unsafe { self.vk_device.free_command_buffers(self.command_pool, &self.command_buffers) };
            self.command_buffers = Self::create_command_buffers(&self.vk_device, self.command_pool, frame_count)?;
        }
        Ok(())
    }

    pub fn queue(&self) -> ash::vk::Queue {
        **self.queue
    }

    pub fn start_recording(&self, index: usize) -> ash::prelude::VkResult<InRecordingCommandBuffer> {
        let command_buffer = self.command_buffers[index];

        let reset_flag = ash::vk::CommandBufferResetFlags::empty();
        unsafe { self.vk_device.reset_command_buffer(command_buffer, reset_flag) }?;

        let begin_info = ash::vk::CommandBufferBeginInfo::default();
        unsafe { self.vk_device.begin_command_buffer(command_buffer, &begin_info) }?;

        Ok(InRecordingCommandBuffer(vulkan::VkRef::shared(&command_buffer)))
    }

    fn create_command_buffers(
        vk_device: &vulkan::VkDeviceHandle,
        command_pool: ash::vk::CommandPool,
        frame_count: u32,
    ) -> Result<Vec<ash::vk::CommandBuffer>, crate::ScError> {
        let allocate_info = ash::vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(ash::vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(frame_count);

        Ok(unsafe { vk_device.allocate_command_buffers(&allocate_info)? })
    }
}

impl Drop for CommandInterface {
    fn drop(&mut self) {
        unsafe { self.vk_device.destroy_command_pool(self.command_pool, None) }
    }
}

pub struct InRecordingCommandBuffer(vulkan::VkRef<ash::vk::CommandBuffer>);

impl std::ops::Deref for InRecordingCommandBuffer {
    type Target = ash::vk::CommandBuffer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl InRecordingCommandBuffer {
    pub fn end_recording(self, device: &ash::Device) -> ash::prelude::VkResult<RecordedCommandBuffer> {
        let command_buffer = self.0;
        unsafe { device.end_command_buffer(*command_buffer) }?;
        Ok(RecordedCommandBuffer(command_buffer))
    }
}

pub struct RecordedCommandBuffer(vulkan::VkRef<ash::vk::CommandBuffer>);

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
