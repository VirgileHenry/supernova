mod render_pass;
mod shaders;

use crate::propellant::vulkan;

use render_pass::color_pass;
use render_pass::geometry_pass;
use render_pass::post_processing_pass;

pub struct Renderer {
    /// Reference to the event loop proxy to send messages to the engine.
    event_loop_proxy: crate::propellant::EventLoopProxy,

    /// Handle to the vulkan device to use the vulkan API.
    vk_device: vulkan::VkDeviceHandle,
    /// Vulkan swapchain interface.
    swapchain: vulkan::VkSwapchain,
    /// Vulkan synchronization interface.
    synchronization: vulkan::VkSynchronization,
    /// Vulkan command interface for the graphics part.
    graphics_command: vulkan::CommandInterface,

    /// Geometry render pass of the scene.
    geometry_pass: geometry_pass::GeometryPass,
    /// Color render pass on top of the G-buffer.
    color_pass: color_pass::ColorPass,
    /// Post processing pass after the other rendering passes.
    post_proc_pass: post_processing_pass::PostProcessingPass,

    /// Targets for the geometry render pass.
    /// One per image in the swapchain.
    geometry_pass_targets: Vec<render_pass::RenderPassTarget>,
}

impl Renderer {
    pub fn create(
        vk_context: &crate::propellant::VkInstance,
        vk_device: crate::propellant::vulkan::VkDeviceHandle,
        window: &winit::window::Window,
        event_loop_proxy: crate::propellant::EventLoopProxy,
    ) -> Result<Renderer, crate::ScError> {
        let swapchain = vulkan::VkSwapchain::create(vk_context, &vk_device, window, None)?;
        let synchronization = vulkan::VkSynchronization::create(&vk_device, swapchain.image_count())?;

        let geometry_pass = geometry_pass::GeometryPass::create(&vk_device, &swapchain)?;

        let image_views = swapchain.image_views();
        let geometry_pass_targets = image_views
            .into_iter()
            .map(|image| {
                let attachments = [image];
                render_pass::RenderPassTarget::create(&vk_device, geometry_pass.render_pass(), &attachments, swapchain.extent())
            })
            .collect::<Result<Vec<_>, _>>()?;

        let frame_count = swapchain.image_count() as u32;
        let graphics_command = vulkan::CommandInterface::create(&vk_device, vk_device.graphics_queue(), frame_count)?;

        Ok(Renderer {
            event_loop_proxy,
            vk_device,
            swapchain,
            synchronization,
            graphics_command,
            geometry_pass,
            color_pass: color_pass::ColorPass,
            post_proc_pass: post_processing_pass::PostProcessingPass,
            geometry_pass_targets,
        })
    }

    pub fn render(&mut self, world: &hecs::World) -> ash::prelude::VkResult<()> {
        /* Wait for the frame slot to be usable,  */
        self.synchronization.frame().wait_until_available()?;

        /* Get the next image from the swapchain */
        let image_available = self.synchronization.frame().image_available;
        let acquire = self.swapchain.acquire_next_image(image_available)?;
        let image_index = acquire.image_index;

        /* Wait for any frame using this image to terminate */
        self.synchronization.claim_image(image_index)?;

        let mut geometry_pass_output = geometry_pass::GeometryPassOutput {
            target: &self.geometry_pass_targets[image_index],
        };

        let command_buffer = self.graphics_command.start_recording(image_index)?;

        let geometry_pass_input = geometry_pass::GeometryPassInput {
            command_buffer,
            render_area: ash::vk::Rect2D {
                offset: ash::vk::Offset2D::default(), // 0, 0
                extent: self.swapchain.extent(),
            },
        };

        use render_pass::RenderingPass;
        self.geometry_pass
            .render(world, &self.vk_device, &geometry_pass_input, &mut geometry_pass_output);

        /*
        self.color_pass.render(world, &self.vulkan_interface, &(), &mut ());
        self.post_proc_pass.render(world, &self.vulkan_interface, &(), &mut ());
        */

        let recorded_command_buffer = geometry_pass_input.command_buffer.end_recording(&self.vk_device)?;

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
            &self.vk_device,
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
            Err(ash::vk::Result::SUBOPTIMAL_KHR)
        } else {
            Ok(())
        }
    }

    pub fn recreate(&mut self, vulkan_state: &vulkan::VkInstance, window: &winit::window::Window) -> Result<(), crate::ScError> {
        if let Err(e) = unsafe { self.vk_device.device_wait_idle() } {
            log::warn!("Failed to wait device idle before recreating swapchain: {e}");
        }

        let new_swapchain = vulkan::VkSwapchain::create(vulkan_state, &self.vk_device, window, Some(&self.swapchain))?;
        self.swapchain = new_swapchain;
        self.geometry_pass = geometry_pass::GeometryPass::create(&self.vk_device, &self.swapchain)?;

        let image_views = self.swapchain.image_views();
        self.geometry_pass_targets = image_views
            .into_iter()
            .map(|image| {
                let attachments = [image];
                render_pass::RenderPassTarget::create(
                    &self.vk_device,
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
}

impl crate::propellant::System for Renderer {
    fn name(&self) -> &'static str {
        "Renderer"
    }

    fn frequency(&self) -> crate::propellant::UpdateFrequency {
        crate::propellant::UpdateFrequency::PerFrame
    }

    fn update(&mut self, world: &mut hecs::World, _: std::time::Duration) {
        match self.render(world) {
            Ok(_) => {}
            Err(ash::vk::Result::SUBOPTIMAL_KHR) => {
                log::info!("Swapchain returned suboptimal KHR, asking for recreation");
                let event = crate::propellant::EngineEvent::SwapchainRecreationRequest;
                if let Err(e) = self.event_loop_proxy.send_event(event) {
                    log::warn!("Failed to send swapchain recreation event: {e}");
                }
            }
            Err(ash::vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                log::info!("Swapchain returned out of date KHR, asking for recreation");
                let event = crate::propellant::EngineEvent::SwapchainRecreationRequest;
                if let Err(e) = self.event_loop_proxy.send_event(event) {
                    log::warn!("Failed to send swapchain recreation event: {e}");
                }
            }
            Err(e) => log::warn!("Error while rendering world: {e}"),
        }
    }

    fn handle_event(&mut self, event: crate::propellant::SystemEvent) {
        match event {
            crate::propellant::SystemEvent::SwapchainRecreationRequest { vulkan_state, window } => {
                match self.recreate(vulkan_state, window) {
                    Ok(_) => {}
                    Err(e) => log::warn!("Failed to recreate swapchain: {e}"),
                }
            }
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        if let Err(e) = unsafe { self.vk_device.device_wait_idle() } {
            log::warn!("Failed to wait device idle for cleanup: {e}");
        }
    }
}
