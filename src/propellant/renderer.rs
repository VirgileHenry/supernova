mod render_pass;
mod shaders;
mod texture;

use crate::propellant::vulkan;

use render_pass::color_pass;
use render_pass::geometry_pass;
use render_pass::post_processing_pass;

pub struct Renderer {
    /// Renderer state
    event_loop_proxy: crate::propellant::EventLoopProxy,
    vulkan_interface: vulkan::VkRendererInterface,
    swapchain: vulkan::Swapchain,
    graphics_command: vulkan::CommandInterface,
    /// All render passes of the renderer
    geometry_pass: geometry_pass::GeometryPass,
    color_pass: color_pass::ColorPass,
    post_proc_pass: post_processing_pass::PostProcessingPass,
    /// All targets for our render passes
    geometry_pass_targets: Vec<render_pass::RenderPassTarget>,
}

impl Renderer {
    pub fn create(
        vulkan_state: &vulkan::VulkanState,
        window: &winit::window::Window,
        event_loop_proxy: crate::propellant::EventLoopProxy,
    ) -> Result<Renderer, crate::ScError> {
        let vulkan_interface = vulkan_state.vulkan_interface.renderer_interface();
        let swapchain = vulkan::Swapchain::create(vulkan_state, &vulkan_interface, window)?;

        let geometry_pass = geometry_pass::GeometryPass::create(&vulkan_interface, &swapchain)?;

        let geometry_pass_targets = swapchain
            .image_views()
            .map(|image| {
                let attachments = [*image];
                render_pass::RenderPassTarget::create(
                    &vulkan_interface,
                    geometry_pass.render_pass(),
                    &attachments,
                    swapchain.extent(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let frame_count = swapchain.frame_count() as u32;
        let graphics_command =
            vulkan::CommandInterface::create(&vulkan_interface, vulkan_interface.graphics_queue(), frame_count)?;

        Ok(Renderer {
            event_loop_proxy,
            vulkan_interface,
            swapchain,
            graphics_command,
            geometry_pass,
            color_pass: color_pass::ColorPass,
            post_proc_pass: post_processing_pass::PostProcessingPass,
            geometry_pass_targets,
        })
    }

    pub fn render(&mut self, world: &hecs::World) {
        // call the different render passes, piping the in/out accordingly

        let (image_index, sync) = match self.swapchain.go_to_next_frame(&self.vulkan_interface) {
            Ok(res) => res,
            Err(crate::ScError::Vulkan(ash::vk::Result::ERROR_OUT_OF_DATE_KHR)) => {
                log::debug!("Swapchain returned out of date KHR, asking for recreation");
                match self
                    .event_loop_proxy
                    .send_event(crate::propellant::EngineEvent::SwapchainRecreationRequest)
                {
                    Ok(_) => {}
                    Err(e) => log::warn!("Failed to send SwapchainRecreationRequest event: {e}"),
                }
                return;
            }
            Err(e) => {
                log::warn!("Unable to acquire next swapchain image index: {e}, can't render");
                return;
            }
        };

        let mut geometry_pass_output = match self.geometry_pass_targets.get_mut(image_index) {
            Some(target) => geometry_pass::GeometryPassOutput { target: *target },
            None => {
                log::warn!("Invalid image index {image_index}, out of bounds for render targets, can't render");
                return;
            }
        };

        let geometry_pass_input = geometry_pass::GeometryPassInput {
            command_buffer: self.graphics_command.start_recording(&self.vulkan_interface, image_index),
            render_area: ash::vk::Rect2D {
                offset: ash::vk::Offset2D::default(), // 0, 0
                extent: self.swapchain.extent(),
            },
        };

        use render_pass::RenderingPass;
        self.geometry_pass
            .render(world, &self.vulkan_interface, &geometry_pass_input, &mut geometry_pass_output);

        /*
        self.color_pass.render(world, &self.vulkan_interface, &(), &mut ());
        self.post_proc_pass.render(world, &self.vulkan_interface, &(), &mut ());
        */

        let recorded_command_buffer = geometry_pass_input.command_buffer.end_recording(&self.vulkan_interface);

        let wait_semaphores = [sync.image_available];
        // shall be the same size as wait_semaphore
        let wait_stages = [ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [*recorded_command_buffer];
        let signal_semaphores = [sync.render_finished];

        let submit_infos = [ash::vk::SubmitInfo {
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: if wait_semaphores.is_empty() {
                std::ptr::null()
            } else {
                wait_semaphores.as_ptr()
            },
            p_wait_dst_stage_mask: if wait_semaphores.is_empty() {
                std::ptr::null()
            } else {
                wait_stages.as_ptr()
            },
            command_buffer_count: command_buffers.len() as u32,
            p_command_buffers: if command_buffers.is_empty() {
                std::ptr::null()
            } else {
                command_buffers.as_ptr()
            },
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: if signal_semaphores.is_empty() {
                std::ptr::null()
            } else {
                signal_semaphores.as_ptr()
            },
            ..Default::default()
        }];

        match recorded_command_buffer.queue_submit(
            &self.vulkan_interface,
            self.vulkan_interface.graphics_queue().queue,
            &submit_infos,
            sync.frame_finished,
        ) {
            Ok(_) => {}
            Err(e) => log::warn!("Failed to submit command buffer to queue! {e}"),
        }

        match self
            .swapchain
            .present(image_index, sync.render_finished, **self.vulkan_interface.present_queue())
        {
            Ok(true) => {
                log::debug!("Swapchain returned suboptimal KHR, asking for recreation");
                match self
                    .event_loop_proxy
                    .send_event(crate::propellant::EngineEvent::SwapchainRecreationRequest)
                {
                    Ok(_) => {}
                    Err(e) => log::warn!("Failed to send event: {e}"),
                }
            }
            Ok(_) => {}
            Err(e) => log::warn!("Failed to present image to surface: {e}"),
        }
    }

    pub fn recreate(&mut self, vulkan_state: &vulkan::VulkanState, window: &winit::window::Window) -> Result<(), crate::ScError> {
        unsafe {
            match self.vulkan_interface.device_wait_idle() {
                Ok(_) => {}
                Err(e) => log::warn!("Failed to wait device idle before recreating swapchain: {e}"),
            }
        };

        self.swapchain.destroy_swapchain(&self.vulkan_interface);
        self.swapchain = vulkan::Swapchain::create(vulkan_state, &self.vulkan_interface, window)?;

        self.geometry_pass.destroy(&self.vulkan_interface);
        self.geometry_pass = geometry_pass::GeometryPass::create(&self.vulkan_interface, &self.swapchain)?;

        self.geometry_pass_targets
            .iter_mut()
            .for_each(|target| target.destroy(&self.vulkan_interface));
        self.geometry_pass_targets = self
            .swapchain
            .image_views()
            .map(|image| {
                let attachments = [*image];
                render_pass::RenderPassTarget::create(
                    &self.vulkan_interface,
                    self.geometry_pass.render_pass(),
                    &attachments,
                    self.swapchain.extent(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let frame_count = self.swapchain.frame_count() as u32;
        self.graphics_command
            .recreate_command_buffers(&self.vulkan_interface, frame_count)?;

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
        self.render(world);
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
        match unsafe { self.vulkan_interface.device_wait_idle() } {
            Ok(_) => {}
            Err(e) => log::warn!("Failed to wait device idle for cleanup: {e}"),
        }
        self.graphics_command.destroy(&self.vulkan_interface);
        self.geometry_pass_targets
            .iter_mut()
            .for_each(|target| target.destroy(&self.vulkan_interface));
        self.geometry_pass.destroy(&self.vulkan_interface);
        self.swapchain.destroy_swapchain(&self.vulkan_interface);
    }
}
