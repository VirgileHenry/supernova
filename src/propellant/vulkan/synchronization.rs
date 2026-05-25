use crate::propellant::vulkan;

/// Synchronization primitives for the renderer.
pub struct VkSynchronization {
    /// Reference to the Vulkan device.
    vk_device: vulkan::VkDeviceHandle,

    /// Synchronization primitives for each frame in flight.
    frame_sync: Vec<InFlightFrameSync>,

    /// Synchronization for swapchain images.
    image_sync: Vec<SwapchainImageSync>,

    /// Current frame being rendered.
    ///
    /// This is an index into the frame_sync vector.
    /// It hold wich frame is currently being recorded,
    /// to know which synchronization objects to use.
    current_frame: usize,
}

impl VkSynchronization {
    pub fn create(vk_device: &vulkan::VkDeviceHandle, swapchain_image_count: usize) -> ash::prelude::VkResult<VkSynchronization> {
        let frame_sync: Vec<InFlightFrameSync> = (0..vulkan::MAX_FRAMES_IN_FLIGHTS)
            .map(|_| InFlightFrameSync::create(vk_device))
            .collect::<Result<_, _>>()?;

        let image_sync: Vec<SwapchainImageSync> = (0..swapchain_image_count)
            .map(|_| SwapchainImageSync::create(vk_device))
            .collect::<Result<_, _>>()?;

        Ok(VkSynchronization {
            vk_device: vk_device.clone(),
            frame_sync,
            image_sync,
            current_frame: 0,
        })
    }

    /// Borrow the current frame's sync without advancing.
    /// Use this to drive a single frame; advance only after submit.
    pub fn frame(&self) -> &InFlightFrameSync {
        &self.frame_sync[self.current_frame]
    }

    /// Get the sync for a specific image.
    pub fn image(&self, image_index: usize) -> &SwapchainImageSync {
        &self.image_sync[image_index]
    }

    /// Wait until this image is free of any prior in-flight frame.
    /// Records this frame's fence as the new owner.
    pub fn claim_image(&mut self, image_index: usize) -> ash::prelude::VkResult<()> {
        let current_fence = self.frame().frame_finished;

        /* Ensure the previous frame that was rendering to this image has finished */
        if let Some(prior_fence) = self.image_sync[image_index].in_flight_fence.as_ref() {
            unsafe { self.vk_device.wait_for_fences(&[**prior_fence], true, u64::MAX) }?;
        }

        /* Binds the image with the current frame. */
        self.image_sync[image_index].in_flight_fence = Some(vulkan::VkRef::shared(&current_fence));
        Ok(())
    }

    /// Advance to the next frame slot. Call after submit.
    pub fn advance(&mut self) {
        self.current_frame = (self.current_frame + 1) % vulkan::MAX_FRAMES_IN_FLIGHTS;
    }
}

/// Set of primitives to perform synchronization on a frame in flight.
///
/// Synchronization tied to a CPU-side "frame slot", one of N concurrent
/// recording contexts that the CPU cycles through.
#[derive(Debug, Clone)]
pub struct InFlightFrameSync {
    /// Reference to the Vulkan device.
    vk_device: vulkan::VkDeviceHandle,

    /// Signaled by the GPU when the acquired swapchain image is actually
    /// available for rendering. Waited on by the graphics submit.
    pub image_available: ash::vk::Semaphore,

    /// Signaled by the GPU when this frame's submit completes.
    /// Waited on by the CPU before reusing this frame slot.
    frame_finished: ash::vk::Fence,
}

impl InFlightFrameSync {
    pub fn create(vk_device: &vulkan::VkDeviceHandle) -> ash::prelude::VkResult<Self> {
        let semaphore_create_info = ash::vk::SemaphoreCreateInfo::default();
        let fence_create_info = ash::vk::FenceCreateInfo::default().flags(ash::vk::FenceCreateFlags::SIGNALED);
        Ok(Self {
            vk_device: vk_device.clone(),
            image_available: unsafe { vk_device.create_semaphore(&semaphore_create_info, None)? },
            frame_finished: unsafe { vk_device.create_fence(&fence_create_info, None)? },
        })
    }

    /// Block until this frame slot's previous submit is complete
    pub fn wait_until_available(&self) -> ash::prelude::VkResult<()> {
        unsafe { self.vk_device.wait_for_fences(&[self.frame_finished], true, u64::MAX) }
    }

    /// Reset the frame finished fence to be signaled again.
    pub fn reset_frame_for_submit(&self) -> ash::prelude::VkResult<ash::vk::Fence> {
        unsafe { self.vk_device.reset_fences(&[self.frame_finished]) }?;
        Ok(self.frame_finished)
    }
}

impl Drop for InFlightFrameSync {
    fn drop(&mut self) {
        unsafe {
            self.vk_device.destroy_semaphore(self.image_available, None);
            self.vk_device.destroy_fence(self.frame_finished, None);
        }
    }
}

/// Synchronization tied to a specific swapchain image.
///
/// Recreated when the swapchain is recreated, because count depends on
/// swapchain image count.
#[derive(Debug, Clone)]
pub struct SwapchainImageSync {
    /// Reference to the Vulkan device.
    vk_device: vulkan::VkDeviceHandle,

    /// Signaled by the graphics submit when rendering to this image is done.
    /// Waited on by the presentation engine before showing the image.
    ///
    /// Must be per-image because the presentation engine may hold onto it
    /// across multiple frame slots, and we can't safely reuse it until the
    /// image is re-acquired.
    pub render_finished: ash::vk::Semaphore,

    /// Non-owning reference to whichever frame's fence is currently
    /// rendering to this image. None if this image hasn't been used yet
    /// since swapchain creation.
    ///
    /// Used to wait for the prior frame to finish before we reuse the image.
    pub in_flight_fence: Option<vulkan::VkRef<ash::vk::Fence>>,
}

impl SwapchainImageSync {
    pub fn create(vk_device: &vulkan::VkDeviceHandle) -> ash::prelude::VkResult<Self> {
        let semaphore_create_info = ash::vk::SemaphoreCreateInfo::default();
        Ok(Self {
            vk_device: vk_device.clone(),
            render_finished: unsafe { vk_device.create_semaphore(&semaphore_create_info, None)? },
            in_flight_fence: None,
        })
    }
}

impl Drop for SwapchainImageSync {
    fn drop(&mut self) {
        unsafe {
            self.vk_device.destroy_semaphore(self.render_finished, None);
        }
    }
}
