use crate::propellant::renderer::shaders;
use crate::propellant::renderer::uniforms;
use crate::propellant::vulkan;

pub struct GeometryPassInput {
    /// Index of the current in flight frame.
    /// Guaranteed to be in `[0, vulkan::MAX_FRAMES_IN_FLIGHTS[`.
    pub frame_index: usize,
    pub command_buffer: vulkan::InRecordingCommandBuffer,
    pub render_area: ash::vk::Rect2D,
}

pub struct GeometryPassOutput<'output> {
    pub target: &'output super::RenderPassTarget,
}

/// For now, the only render pass that renders super simple geometry to test architecture
pub struct GeometryPass {
    /// Handle to the vk device
    vk_device: vulkan::VkDeviceHandle,
    /// Pipeline layout object
    pipeline_layout: ash::vk::PipelineLayout,
    /// Pipeline object
    pipeline: ash::vk::Pipeline,
    /// Render pass object
    render_pass: ash::vk::RenderPass,

    /// Descriptor set layout
    descriptor_set_layout: ash::vk::DescriptorSetLayout,
    /// Descriptor pool
    descriptor_pool: ash::vk::DescriptorPool,
    /// Descriptor sets, one per frame in flight
    descriptor_sets: [ash::vk::DescriptorSet; vulkan::MAX_FRAMES_IN_FLIGHT],

    /// Uniform buffer for the camera uniform.
    camera_uniform: vulkan::RingUniformBuffer<uniforms::CameraUniform, { vulkan::MAX_FRAMES_IN_FLIGHT }>,
    /// Storage buffer for the segments instances
    instance_buffer: vulkan::RingInstanceBuffer<uniforms::SegmentInstanceData, { vulkan::MAX_FRAMES_IN_FLIGHT }>,
}

impl GeometryPass {
    pub fn create(vk_device: &vulkan::VkDeviceHandle, swapchain: &vulkan::VkSwapchain) -> Result<GeometryPass, vulkan::VkError> {
        /* Fixme: PLEASE refactor me */

        const VERTEX_SHADER: shaders::ShaderSource = shaders::SEGMENT_VERT;
        const FRAGMENT_SHADER: shaders::ShaderSource = shaders::SEGMENT_FRAG;

        let vertex_shader = vk_device.load_shader_module(VERTEX_SHADER.code)?;
        let fragment_shader = vk_device.load_shader_module(FRAGMENT_SHADER.code)?;

        let vert_stage = ash::vk::PipelineShaderStageCreateInfo::default()
            .stage(ash::vk::ShaderStageFlags::VERTEX)
            .module(vertex_shader)
            .name(VERTEX_SHADER.entry_point);

        let frag_stage = ash::vk::PipelineShaderStageCreateInfo::default()
            .stage(ash::vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_shader)
            .name(FRAGMENT_SHADER.entry_point);

        let vertex_input_state = ash::vk::PipelineVertexInputStateCreateInfo::default();
        let input_assembly_state = ash::vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(ash::vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

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

        let rasterization_state = ash::vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(ash::vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(ash::vk::CullModeFlags::BACK)
            .depth_bias_enable(false)
            .front_face(ash::vk::FrontFace::CLOCKWISE);

        let multisample_state = ash::vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(ash::vk::SampleCountFlags::TYPE_1);

        let attachment = ash::vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(ash::vk::ColorComponentFlags::RGBA)
            .blend_enable(false);
        let attachments = [attachment];

        let color_blend_state = ash::vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(ash::vk::LogicOp::COPY)
            .attachments(&attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let bindings = [
            ash::vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(ash::vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(ash::vk::ShaderStageFlags::VERTEX),
            ash::vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(ash::vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(ash::vk::ShaderStageFlags::VERTEX),
        ];

        let descriptor_set_layout_info = ash::vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let descriptor_set_layout = unsafe { vk_device.create_descriptor_set_layout(&descriptor_set_layout_info, None)? };
        let set_layouts = [descriptor_set_layout];

        let pipeline_layout_info = ash::vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts);
        let pipeline_layout = unsafe { vk_device.create_pipeline_layout(&pipeline_layout_info, None)? };

        let render_pass = Self::create_render_pass(vk_device, swapchain)?;
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

        unsafe { vk_device.destroy_shader_module(vertex_shader, None) };
        unsafe { vk_device.destroy_shader_module(fragment_shader, None) };

        let cam_uniform_capacity = std::num::NonZeroUsize::new(1).unwrap();
        let camera_uniform = vulkan::RingUniformBuffer::create(&vk_device, cam_uniform_capacity)?;

        let instance_buffer_capacity = std::num::NonZeroUsize::new(128).unwrap();
        let instance_buffer = vulkan::RingInstanceBuffer::create(&vk_device, instance_buffer_capacity)?;

        let pool_sizes = [
            ash::vk::DescriptorPoolSize::default()
                .ty(ash::vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(vulkan::MAX_FRAMES_IN_FLIGHT as u32),
            ash::vk::DescriptorPoolSize::default()
                .ty(ash::vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(vulkan::MAX_FRAMES_IN_FLIGHT as u32),
        ];

        let pool_info = ash::vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(vulkan::MAX_FRAMES_IN_FLIGHT as u32);

        let descriptor_pool = unsafe { vk_device.create_descriptor_pool(&pool_info, None)? };

        let descriptor_set_layouts = [descriptor_set_layout; vulkan::MAX_FRAMES_IN_FLIGHT];
        let alloc_info = ash::vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&descriptor_set_layouts);

        let descriptor_sets: [ash::vk::DescriptorSet; vulkan::MAX_FRAMES_IN_FLIGHT] = unsafe {
            vk_device
                .allocate_descriptor_sets(&alloc_info)?
                .try_into()
                .expect("Descriptor set count mismatch")
        };

        for frame_index in 0..vulkan::MAX_FRAMES_IN_FLIGHT {
            let camera_info = camera_uniform.get(frame_index).descriptor_info();
            let instance_info = instance_buffer.get(frame_index).descriptor_info();

            let camera_infos = [camera_info];
            let instance_infos = [instance_info];

            let writes = [
                ash::vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_sets[frame_index])
                    .dst_binding(0)
                    .descriptor_type(ash::vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&camera_infos),
                ash::vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_sets[frame_index])
                    .dst_binding(1)
                    .descriptor_type(ash::vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(&instance_infos),
            ];

            unsafe { vk_device.update_descriptor_sets(&writes, &[]) }
        }

        Ok(GeometryPass {
            vk_device: vk_device.clone(),
            pipeline_layout,
            pipeline,
            render_pass,
            descriptor_pool,
            descriptor_sets,
            descriptor_set_layout,
            camera_uniform,
            instance_buffer,
        })
    }

    pub fn render_pass(&self) -> ash::vk::RenderPass {
        self.render_pass
    }

    fn create_render_pass(
        vk_device: &vulkan::VkDeviceHandle,
        swapchain: &vulkan::VkSwapchain,
    ) -> Result<ash::vk::RenderPass, vulkan::VkError> {
        let color_attachment = ash::vk::AttachmentDescription::default()
            .format(swapchain.format())
            .samples(ash::vk::SampleCountFlags::TYPE_1)
            .load_op(ash::vk::AttachmentLoadOp::CLEAR)
            .store_op(ash::vk::AttachmentStoreOp::STORE)
            .stencil_load_op(ash::vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(ash::vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(ash::vk::ImageLayout::UNDEFINED)
            .final_layout(ash::vk::ImageLayout::PRESENT_SRC_KHR)
            .flags(ash::vk::AttachmentDescriptionFlags::empty());

        let color_attachment_ref = ash::vk::AttachmentReference::default()
            .attachment(0)
            .layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let color_attachments = [color_attachment_ref];

        let subpass = ash::vk::SubpassDescription::default()
            .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments);

        let subpass_dependency = ash::vk::SubpassDependency::default()
            .src_subpass(ash::vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(ash::vk::AccessFlags::empty())
            .dst_stage_mask(ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let attachments = [color_attachment];
        let subpasses = [subpass];
        let dependencies = [subpass_dependency];

        let create_info = ash::vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        Ok(unsafe { vk_device.create_render_pass(&create_info, None)? })
    }
}

impl super::RenderingPass for GeometryPass {
    type Input<'input> = GeometryPassInput;
    type Output<'output> = GeometryPassOutput<'output>;

    fn render<'input, 'output>(
        &mut self,
        vk_device: &vulkan::VkDeviceHandle,
        assets: &crate::propellant::assets::AssetManager,
        world: &hecs::World,
        input: &Self::Input<'input>,
        out: &mut Self::Output<'output>,
    ) -> ash::prelude::VkResult<()> {
        use crate::propellant::components::*;

        let mut camera_query = world.query::<(&Transform, &Camera<camera::MainCamera>)>();
        let Some((_, (transform, camera))) = camera_query.iter().next() else {
            log::warn!("No main camera found, skipping geometry pass");
            return Ok(());
        };
        /* Set the main camera uniforms */
        let camera_uniform = uniforms::CameraUniform::from_camera_components(transform, camera);
        self.camera_uniform.get_mut(input.frame_index).write(&camera_uniform);

        /* Begin render pass */
        let render_pass_begin = ash::vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(out.target.framebuffer)
            .render_area(input.render_area)
            .clear_values(&[ash::vk::ClearValue {
                color: ash::vk::ClearColorValue {
                    float32: [0., 0., 0., 1.],
                },
            }]);

        unsafe { vk_device.cmd_begin_render_pass(*input.command_buffer, &render_pass_begin, ash::vk::SubpassContents::INLINE) };
        unsafe { vk_device.cmd_bind_pipeline(*input.command_buffer, ash::vk::PipelineBindPoint::GRAPHICS, self.pipeline) };

        let descriptor_set = self.descriptor_sets[input.frame_index];
        let descriptor_sets = [descriptor_set];
        unsafe {
            vk_device.cmd_bind_descriptor_sets(
                *input.command_buffer,
                ash::vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &descriptor_sets,
                &[],
            )
        };

        let mut instance_count = 0;

        let mut query = world.query::<(&Transform, &SegmentRenderer)>();
        for (entity, (transform, segment_renderer)) in query.iter() {
            /* Fixme: maybe the segment component could store the bounding sphere + views to avoid the asset fetch ? */
            let Some(segment) = assets.get_segment(&segment_renderer.handle) else {
                log::warn!(
                    "Dangling segment handle {:?}, unable to render entity",
                    segment_renderer.handle
                );
                continue;
            };
            /* Fixme: culling */

            let instance_data = uniforms::SegmentInstanceData::from_segment_components(transform, segment);
            let instance_buffer = self.instance_buffer.get_mut(input.frame_index);

            instance_buffer.write(instance_count, &instance_data);
            instance_count += 1;
        }
        /* Make a draw call on 6 vertices for each instance */
        unsafe { vk_device.cmd_draw(*input.command_buffer, 6, instance_count as u32, 0, 0) };

        /* End of render pass */
        unsafe { vk_device.cmd_end_render_pass(*input.command_buffer) };

        Ok(())
    }
}

impl Drop for GeometryPass {
    fn drop(&mut self) {
        unsafe {
            self.vk_device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.vk_device.destroy_pipeline(self.pipeline, None);
            self.vk_device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.vk_device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.vk_device.destroy_render_pass(self.render_pass, None);
        }
    }
}
