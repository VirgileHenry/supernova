mod error;
mod render_graph;
mod render_pass;
mod render_target;
mod shaders;

pub use render_graph::Binding;
pub use render_graph::BindingStage;
pub use render_graph::Bindings;
pub use render_graph::RenderGraphInput;
pub use render_graph::RenderGraphOutput;
pub use render_pass::RenderPass;

use crate::vulkan;

pub(crate) struct Renderer<R: render_graph::RenderGraph> {
    /// Reference to the event loop proxy to send messages to the engine.
    event_loop_proxy: crate::event::EventLoopProxy,

    /// Handle to the vulkan device to use the vulkan API.
    vk_device_handle: vulkan::VkDeviceHandle,
    /// Vulkan swapchain interface.
    swapchain: vulkan::VkSwapchain,
    /// Vulkan synchronization interface.
    synchronization: vulkan::VkSynchronization,
    /// Vulkan command interface for the graphics part.
    graphics_command: vulkan::CommandInterface,

    /// Type level render graph
    render_graph: R,

    /// Targets for the geometry render pass.
    /// One per image in the swapchain.
    render_targets: Vec<render_target::RenderTarget>,
    dummy_pass: ash::vk::RenderPass,
}

impl<R: render_graph::RenderGraph> Renderer<R> {
    pub fn create(
        vk_context: &vulkan::VulkanContext,
        event_loop_proxy: crate::event::EventLoopProxy,
        assets: &crate::assets::AssetManager,
    ) -> Result<Self, error::RendererCreateError> {
        let vk_device_handle = vk_context.device_handle();

        let swapchain = vulkan::VkSwapchain::create(vk_context, None)?;
        let synchronization = vulkan::VkSynchronization::create(&vk_device_handle, swapchain.image_count())?;

        let create_context = render_pass::RenderPassCreateContext {
            device: &vk_device_handle,
            swapchain: &swapchain,
            assets,
        };

        let render_graph = R::create(&create_context)?;
        let dummy_pass = create_reference_render_pass(&vk_device_handle, swapchain.format())?;

        let image_views = swapchain.image_views();
        let render_targets = image_views
            .into_iter()
            .map(|image| {
                let attachments = [image];
                render_target::RenderTarget::create(&vk_device_handle, dummy_pass, &attachments, swapchain.extent())
            })
            .collect::<Result<Vec<_>, _>>()?;

        let frame_count = swapchain.image_count() as u32;
        let graphics_queue = vk_device_handle.graphics_queue();
        let graphics_command = vulkan::CommandInterface::create(&vk_device_handle, graphics_queue, frame_count)?;

        log::info!("Created renderer");

        Ok(Renderer {
            event_loop_proxy,
            vk_device_handle,
            swapchain,
            synchronization,
            graphics_command,
            render_graph,
            render_targets,
            dummy_pass,
        })
    }

    pub fn recreate(
        &mut self,
        vk_context: &vulkan::VulkanContext,
        assets: &crate::assets::AssetManager,
    ) -> Result<(), error::RendererCreateError> {
        if let Err(e) = unsafe { self.vk_device_handle.device_wait_idle() } {
            log::warn!("Failed to wait device idle before recreating swapchain: {e}");
        }

        let new_swapchain = vulkan::VkSwapchain::create(vk_context, Some(&self.swapchain))?;
        self.swapchain = new_swapchain;

        let create_context = render_pass::RenderPassCreateContext {
            device: &self.vk_device_handle,
            swapchain: &self.swapchain,
            assets,
        };
        self.render_graph = R::create(&create_context)?;
        self.dummy_pass = create_reference_render_pass(&self.vk_device_handle, self.swapchain.format())?;

        let image_views = self.swapchain.image_views();
        self.render_targets = image_views
            .into_iter()
            .map(|image| {
                let attachments = [image];
                render_target::RenderTarget::create(
                    &self.vk_device_handle,
                    self.dummy_pass,
                    &attachments,
                    self.swapchain.extent(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let frame_count = self.swapchain.image_count() as u32;
        self.graphics_command.recreate_command_buffers(frame_count)?;

        Ok(())
    }

    pub fn render(&mut self, assets: &crate::assets::AssetManager, world: &hecs::World) -> Result<(), error::RenderError> {
        /* Wait for the frame slot to be usable,  */
        self.synchronization.frame().wait_until_available()?;

        /* Get the next image from the swapchain */
        let image_available = self.synchronization.frame().image_available;
        let acquire = self.swapchain.acquire_next_image(image_available)?;
        let image_index = acquire.image_index;
        let frame_index = self.synchronization.frame_index();

        /* Wait for any frame using this image to terminate */
        self.synchronization.claim_image(image_index)?;
        let target = self.render_targets[image_index].framebuffer;

        let render_context = render_pass::RenderPassRenderContext {
            device: &self.vk_device_handle,
            world,
            assets,
            command_buffer: self.graphics_command.start_recording(image_index)?,
            frame_index,
            target,
        };

        let _output = self.render_graph.render(&render_context, render_graph::RenderGraphInput)?;

        let recorded_command_buffer = render_context.command_buffer.end_recording(&self.vk_device_handle)?;

        let wait_semaphores = [self.synchronization.frame().image_available];
        // shall be the same size as wait_semaphore
        let wait_stages = [ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [*recorded_command_buffer];
        let signal_semaphores = [self.synchronization.image(image_index).render_finished];

        let submit_info = ash::vk::SubmitInfo::default()
            .wait_semaphores(wait_semaphores.as_slice())
            .wait_dst_stage_mask(wait_stages.as_slice())
            .command_buffers(command_buffers.as_slice())
            .signal_semaphores(signal_semaphores.as_slice());

        let submit_infos = [submit_info];

        let frame_finished_fence = self.synchronization.frame().reset_frame_for_submit()?;
        match recorded_command_buffer.queue_submit(
            &self.vk_device_handle,
            self.graphics_command.queue(),
            submit_infos.as_slice(),
            frame_finished_fence,
        ) {
            Ok(_) => {}
            Err(e) => log::warn!("Failed to submit command buffer to queue! {e}"),
        }

        let present_ready = self.synchronization.image(image_index).render_finished;
        let present_queue = self.graphics_command.queue();
        self.swapchain.present(image_index, present_ready, present_queue)?;

        self.synchronization.advance();

        if acquire.suboptimal {
            Err(error::RenderError::Vulkan {
                error: ash::vk::Result::SUBOPTIMAL_KHR,
            })
        } else {
            Ok(())
        }
    }
}

impl<R: render_graph::RenderGraph> Drop for Renderer<R> {
    fn drop(&mut self) {
        log::info!("Destroying renderer");
        if let Err(e) = unsafe { self.vk_device_handle.device_wait_idle() } {
            log::warn!("Failed to wait device idle for cleanup: {e}");
        }
    }
}

/// Only used to create framebuffers compatible with the graph's nodes.
/// Fixme: should be removed at some point it creates a dummy render pass just to have framebuffers ?
fn create_reference_render_pass(
    device: &vulkan::VkDeviceHandle,
    format: ash::vk::Format,
) -> ash::prelude::VkResult<ash::vk::RenderPass> {
    let attachment = ash::vk::AttachmentDescription::default()
        .format(format)
        .samples(ash::vk::SampleCountFlags::TYPE_1)
        .load_op(ash::vk::AttachmentLoadOp::DONT_CARE)
        .store_op(ash::vk::AttachmentStoreOp::STORE)
        .initial_layout(ash::vk::ImageLayout::UNDEFINED)
        .final_layout(ash::vk::ImageLayout::PRESENT_SRC_KHR);
    let color_ref = [ash::vk::AttachmentReference::default()
        .attachment(0)
        .layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
    let subpass = [ash::vk::SubpassDescription::default()
        .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_ref)];
    let attachments = [attachment];
    let info = ash::vk::RenderPassCreateInfo::default()
        .attachments(&attachments)
        .subpasses(&subpass);
    unsafe { device.create_render_pass(&info, None) }
}
