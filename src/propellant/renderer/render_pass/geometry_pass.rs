use crate::propellant::renderer::shaders;
use crate::propellant::vulkan;

pub struct GeometryPassInput {
    pub command_buffer: vulkan::InRecordingCommandBuffer,
    pub render_area: ash::vk::Rect2D,
}

pub struct GeometryPassOutput<'output> {
    pub target: &'output super::RenderPassTarget,
}

/// For now, the only render pass that renders super simple geometry to test architecture
pub struct GeometryPass {
    vk_device: vulkan::VkDeviceHandle,
    pipeline_layout: ash::vk::PipelineLayout,
    pipeline: ash::vk::Pipeline,
    render_pass: ash::vk::RenderPass,
}

impl GeometryPass {
    pub fn create(vk_device: &vulkan::VkDeviceHandle, swapchain: &vulkan::VkSwapchain) -> Result<GeometryPass, crate::ScError> {
        let vertex_shader = vk_device.load_shader_module(shaders::EXAMPLE_VERT.code)?;
        let fragment_shader = vk_device.load_shader_module(shaders::EXAMPLE_FRAG.code)?;

        let vert_stage = ash::vk::PipelineShaderStageCreateInfo {
            stage: ash::vk::ShaderStageFlags::VERTEX,
            module: vertex_shader,
            p_name: shaders::EXAMPLE_VERT.entry_point,
            ..Default::default()
        };
        let frag_stage = ash::vk::PipelineShaderStageCreateInfo {
            stage: ash::vk::ShaderStageFlags::FRAGMENT,
            module: fragment_shader,
            p_name: shaders::EXAMPLE_FRAG.entry_point,
            ..Default::default()
        };

        let vertex_input_state = ash::vk::PipelineVertexInputStateCreateInfo { ..Default::default() };
        let input_assembly_state = ash::vk::PipelineInputAssemblyStateCreateInfo {
            topology: ash::vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: ash::vk::FALSE,
            ..Default::default()
        };

        let swapchain_extent = swapchain.extent();
        let viewports = [ash::vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain_extent.width as f32,
            height: swapchain_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [ash::vk::Rect2D {
            offset: ash::vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent,
        }];

        let viewport_state = ash::vk::PipelineViewportStateCreateInfo::default()
            .viewports(viewports.as_slice())
            .scissors(scissors.as_slice());

        let rasterization_state = ash::vk::PipelineRasterizationStateCreateInfo {
            depth_clamp_enable: ash::vk::FALSE,
            rasterizer_discard_enable: ash::vk::FALSE,
            polygon_mode: ash::vk::PolygonMode::FILL,
            line_width: 1.0,
            cull_mode: ash::vk::CullModeFlags::BACK,
            depth_bias_enable: ash::vk::FALSE,
            front_face: ash::vk::FrontFace::CLOCKWISE,
            ..Default::default()
        };

        let multisample_state = ash::vk::PipelineMultisampleStateCreateInfo {
            sample_shading_enable: ash::vk::FALSE,
            rasterization_samples: ash::vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let attachment = ash::vk::PipelineColorBlendAttachmentState {
            color_write_mask: ash::vk::ColorComponentFlags::RGBA,
            blend_enable: ash::vk::FALSE,
            ..Default::default()
        };
        let attachments = [attachment];
        let color_blend_state = ash::vk::PipelineColorBlendStateCreateInfo {
            logic_op_enable: ash::vk::FALSE,
            logic_op: ash::vk::LogicOp::COPY,
            attachment_count: attachments.len() as u32,
            p_attachments: if attachments.is_empty() {
                std::ptr::null()
            } else {
                attachments.as_ptr()
            },
            blend_constants: [0.0, 0.0, 0.0, 0.0],
            ..Default::default()
        };

        let layout_info = ash::vk::PipelineLayoutCreateInfo { ..Default::default() };

        let render_pass = Self::create_render_pass(vk_device, swapchain)?;

        let pipeline_layout = unsafe { vk_device.create_pipeline_layout(&layout_info, None)? };

        let stages = [vert_stage, frag_stage];

        let create_info = ash::vk::GraphicsPipelineCreateInfo::default()
            .stages(stages.as_slice())
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .layout(pipeline_layout)
            .render_pass(render_pass);
        let create_infos = [create_info];

        let pipeline = unsafe { vk_device.create_graphics_pipelines(ash::vk::PipelineCache::null(), &create_infos, None) }
            .map_err(|(_, e)| e)?
            .remove(0);

        log::debug!("Successefuly created geometry render pass");

        unsafe {
            vk_device.destroy_shader_module(vertex_shader, None);
            vk_device.destroy_shader_module(fragment_shader, None);
        }

        Ok(GeometryPass {
            vk_device: vk_device.clone(),
            pipeline_layout,
            pipeline,
            render_pass,
        })
    }

    pub fn render_pass(&self) -> ash::vk::RenderPass {
        self.render_pass
    }

    fn create_render_pass(
        vk_device: &vulkan::VkDeviceHandle,
        swapchain: &vulkan::VkSwapchain,
    ) -> Result<ash::vk::RenderPass, crate::ScError> {
        let color_attachment = ash::vk::AttachmentDescription {
            format: swapchain.format(),
            samples: ash::vk::SampleCountFlags::TYPE_1,
            load_op: ash::vk::AttachmentLoadOp::CLEAR,
            store_op: ash::vk::AttachmentStoreOp::STORE,
            stencil_load_op: ash::vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: ash::vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: ash::vk::ImageLayout::UNDEFINED,
            final_layout: ash::vk::ImageLayout::PRESENT_SRC_KHR,
            flags: ash::vk::AttachmentDescriptionFlags::empty(),
        };

        let color_attachment_ref = ash::vk::AttachmentReference {
            attachment: 0, // layout 0 in shader
            layout: ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let color_attachments = [color_attachment_ref];

        let subpass = ash::vk::SubpassDescription {
            pipeline_bind_point: ash::vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: color_attachments.len() as u32,
            p_color_attachments: if color_attachments.is_empty() {
                std::ptr::null()
            } else {
                color_attachments.as_ptr()
            },
            ..Default::default()
        };

        let subpass_dependency = ash::vk::SubpassDependency {
            src_subpass: ash::vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: ash::vk::AccessFlags::empty(),
            dst_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            ..Default::default()
        };

        let attachments = [color_attachment];
        let subpasses = [subpass];
        let dependencies = [subpass_dependency];

        let create_info = ash::vk::RenderPassCreateInfo::default()
            .attachments(attachments.as_slice())
            .subpasses(subpasses.as_slice())
            .dependencies(dependencies.as_slice());

        Ok(unsafe { vk_device.create_render_pass(&create_info, None)? })
    }
}

impl super::RenderingPass for GeometryPass {
    type Input<'input> = GeometryPassInput;
    type Output<'output> = GeometryPassOutput<'output>;

    fn render<'input, 'output>(
        &self,
        vk_device: &vulkan::VkDeviceHandle,
        assets: &crate::propellant::assets::AssetManager,
        world: &hecs::World,
        input: &Self::Input<'input>,
        out: &mut Self::Output<'output>,
    ) -> ash::prelude::VkResult<()> {
        use crate::propellant::components::*;

        /* Upload camera uniforms ? (support main lights in the future) */

        /* For now, no instancing, let's make draw call for each segment */
        let mut query = world.query::<(&Transform, &SegmentRenderer)>();
        for (entity, (transform, segment_renderer)) in query.iter() {}

        /* Previous rendering code */
        let clear_color = ash::vk::ClearValue {
            color: ash::vk::ClearColorValue {
                float32: [0., 0., 0., 1.],
            },
        };

        let clear_values = [clear_color];

        let render_pass_begin = ash::vk::RenderPassBeginInfo {
            render_pass: self.render_pass,
            framebuffer: out.target.framebuffer,
            render_area: input.render_area,
            clear_value_count: clear_values.len() as u32,
            p_clear_values: if clear_values.is_empty() {
                std::ptr::null()
            } else {
                clear_values.as_ptr()
            },
            ..Default::default()
        };

        unsafe {
            vk_device.cmd_begin_render_pass(*input.command_buffer, &render_pass_begin, ash::vk::SubpassContents::INLINE);
            vk_device.cmd_bind_pipeline(*input.command_buffer, ash::vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            vk_device.cmd_draw(*input.command_buffer, 3, 1, 0, 0);
            vk_device.cmd_end_render_pass(*input.command_buffer);
        };
        /* End of Previous rendering code */

        Ok(())
    }
}

impl Drop for GeometryPass {
    fn drop(&mut self) {
        unsafe {
            self.vk_device.destroy_pipeline(self.pipeline, None);
            self.vk_device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.vk_device.destroy_render_pass(self.render_pass, None);
        }
    }
}
