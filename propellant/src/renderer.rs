mod error;
mod render_graph;
mod render_pass;
mod render_target;
mod shaders;
mod uniforms;

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
}

impl<R: render_graph::RenderGraph> Renderer<R> {
    pub fn create(
        vk_context: &vulkan::VulkanContext,
        event_loop_proxy: crate::event::EventLoopProxy,
        assets: &crate::assets::AssetManager,
    ) -> Result<Self, vulkan::VkError> {
        let vk_device_handle = vk_context.device_handle();

        let swapchain = vulkan::VkSwapchain::create(vk_context, None)?;
        let synchronization = vulkan::VkSynchronization::create(&vk_device_handle, swapchain.image_count())?;

        let render_graph = R::create()?;

        let image_views = swapchain.image_views();
        let render_targets = image_views
            .into_iter()
            .map(|image| {
                let attachments = [image];
                render_target::RenderTarget::create(
                    &vk_device_handle,
                    geometry_pass.render_pass(),
                    &attachments,
                    swapchain.extent(),
                )
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
        })
    }

    pub fn recreate(
        &mut self,
        vk_context: &vulkan::VulkanContext,
        assets: &crate::assets::AssetManager,
    ) -> Result<(), vulkan::VkError> {
        if let Err(e) = unsafe { self.vk_device_handle.device_wait_idle() } {
            log::warn!("Failed to wait device idle before recreating swapchain: {e}");
        }

        let new_swapchain = vulkan::VkSwapchain::create(vk_context, Some(&self.swapchain))?;
        self.swapchain = new_swapchain;

        self.render_graph = R::create()?;

        let image_views = self.swapchain.image_views();
        self.render_targets = image_views
            .into_iter()
            .map(|image| {
                let attachments = [image];
                render_target::RenderTarget::create(
                    &self.vk_device_handle,
                    self.geometry_pass.render_pass(),
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

        let render_graph_input = render_graph::RenderGraphInput {
            frame_index,
            command_buffer: self.graphics_command.start_recording(image_index)?,
            render_area: ash::vk::Rect2D {
                offset: ash::vk::Offset2D::default(),
                extent: self.swapchain.extent(),
            },
        };

        let render_graph_output = self
            .render_graph
            .render(&self.vk_device_handle, assets, world, render_graph_input)?;

        let recorded_command_buffer = render_graph_output.command_buffer.end_recording(&self.vk_device_handle)?;

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

impl<R: render_graph::RenderGraph> crate::System for Renderer {
    fn name(&self) -> &'static str {
        "Renderer"
    }

    fn frequency(&self) -> crate::UpdateFrequency {
        crate::UpdateFrequency::PerFrame
    }

    fn update(&mut self, assets: &crate::assets::AssetManager, world: &mut hecs::World, _: std::time::Duration) {
        match self.render(assets, world) {
            Ok(_) => {}
            Err(error::RenderError::Vulkan {
                error: ash::vk::Result::SUBOPTIMAL_KHR,
            }) => {
                log::debug!("Swapchain returned suboptimal KHR, asking for recreation");
                let event = crate::EngineEvent::SwapchainRecreationRequest;
                if let Err(e) = self.event_loop_proxy.send_event(event) {
                    log::warn!("Failed to send swapchain recreation event: {e}");
                }
            }
            Err(error::RenderError::Vulkan {
                error: ash::vk::Result::ERROR_OUT_OF_DATE_KHR,
            }) => {
                log::debug!("Swapchain returned out of date KHR, asking for recreation");
                let event = crate::EngineEvent::SwapchainRecreationRequest;
                if let Err(e) = self.event_loop_proxy.send_event(event) {
                    log::warn!("Failed to send swapchain recreation event: {e}");
                }
            }
            Err(e) => log::warn!("Error while rendering world: {e}"),
        }
    }

    fn handle_event(&mut self, world: &mut hecs::World, event: crate::SystemEvent) {
        match event {
            crate::SystemEvent::SwapchainRecreationRequest { vk_context, assets } => match self.recreate(vk_context, assets) {
                Ok(_) => {}
                Err(e) => log::warn!("Failed to recreate swapchain: {e}"),
            },
            crate::SystemEvent::WindowResize { width, height } => {
                use crate::components;
                let mut query = world.query::<&mut components::Camera<components::camera::MainCamera>>();
                for (_, main_camera) in query.iter() {
                    main_camera.resize(width, height);
                }
            }
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
