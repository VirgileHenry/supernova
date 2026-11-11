use crate::renderer::Bindings;
use crate::renderer::render_graph::RenderGraphImpl;
use crate::renderer::render_pass::RenderPass;
use crate::renderer::render_pass::RenderPassCreateContext;
use crate::renderer::render_pass::RenderPassRenderContext;
use crate::vulkan::MAX_FRAMES_IN_FLIGHT;
use ash::prelude::VkResult;

pub struct RenderGraphNode<Pass: RenderPass> {
    /// Handle to the vk device
    vk_device: crate::vulkan::VkDeviceHandle,
    /// Pipeline layout object
    pipeline_layout: ash::vk::PipelineLayout,
    /// Pipeline object
    pipeline: ash::vk::Pipeline,
    /// Render pass object
    vk_render_pass: ash::vk::RenderPass,

    /// Actual user defined render pass
    user_render_pass: Pass,

    /// Render pass descriptors objects
    descriptors: RenderPassDescriptors<Pass::Bindings>,
}

impl<Pass: RenderPass> RenderGraphImpl for RenderGraphNode<Pass> {
    type Input = Pass::Input;
    type Output = Pass::Output;

    type CreateError = NodeCreateError<Pass::Bindings>;

    fn create(context: &RenderPassCreateContext) -> Result<Self, Self::CreateError> {
        let device = context.device;
        let swapchain = context.swapchain;

        let vertex_shader = device.load_shader_module(Pass::VERT_SHADER.code)?;
        let fragment_shader = device.load_shader_module(Pass::FRAG_SHADER.code)?;

        let vert_stage = ash::vk::PipelineShaderStageCreateInfo::default()
            .stage(ash::vk::ShaderStageFlags::VERTEX)
            .module(vertex_shader)
            .name(Pass::VERT_SHADER.entry_point);

        let frag_stage = ash::vk::PipelineShaderStageCreateInfo::default()
            .stage(ash::vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_shader)
            .name(Pass::FRAG_SHADER.entry_point);

        let vertex_input_state = ash::vk::PipelineVertexInputStateCreateInfo::default();
        let input_assembly_state = ash::vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(ash::vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let swapchain_extent = context.swapchain.extent();
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

        let descriptors = RenderPassDescriptors::create(context)?;

        let set_layouts = [descriptors.descriptor_set_layout];

        let pipeline_layout_info = ash::vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts);
        let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None)? };

        let vk_render_pass = create_render_pass(device, swapchain)?;
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
            .render_pass(vk_render_pass);
        let create_infos = [create_info];

        let pipeline = unsafe { device.create_graphics_pipelines(ash::vk::PipelineCache::null(), &create_infos, None) }
            .map_err(|(_, e)| e)?
            .remove(0);

        log::debug!("Created \"{}\" render pass", Pass::NAME);

        unsafe { device.destroy_shader_module(vertex_shader, None) };
        unsafe { device.destroy_shader_module(fragment_shader, None) };

        let user_render_pass = Pass::create();

        Ok(Self {
            vk_device: device.clone(),
            pipeline_layout,
            pipeline,
            vk_render_pass,
            user_render_pass,
            descriptors,
        })
    }

    fn render(&mut self, context: &RenderPassRenderContext, input: Self::Input) -> VkResult<Self::Output> {
        let device = context.device;
        let cmd = *context.command_buffer;

        /* 1. update the bindings' data for this frame (camera, instances...) */
        self.descriptors.bindings.update(context)?;

        /* 2. begin the render pass */
        let clear_values = [ash::vk::ClearValue {
            color: ash::vk::ClearColorValue {
                float32: [0., 0., 0., 1.],
            },
        }];

        let begin_info = ash::vk::RenderPassBeginInfo::default()
            .render_pass(self.vk_render_pass)
            .framebuffer(context.target)
            .render_area(context.render_area)
            .clear_values(&clear_values);

        let descriptor_sets = [self.descriptors.descriptor_sets[context.frame_index]];

        unsafe {
            device.cmd_begin_render_pass(cmd, &begin_info, ash::vk::SubpassContents::INLINE);
            /* 3. pipeline + this frame's descriptor set */
            device.cmd_bind_pipeline(cmd, ash::vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            device.cmd_bind_descriptor_sets(
                cmd,
                ash::vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &descriptor_sets,
                &[],
            );
        }

        /* 4. the user pass: draws only */
        let output = self.user_render_pass.render(context, &self.descriptors.bindings, input);

        /* 5. always end the render pass, even if the user pass returned an error */
        unsafe { device.cmd_end_render_pass(cmd) };

        output
    }
}

impl<Pass: RenderPass> Drop for RenderGraphNode<Pass> {
    fn drop(&mut self) {
        unsafe {
            self.vk_device.destroy_pipeline(self.pipeline, None);
            self.vk_device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.vk_device.destroy_render_pass(self.vk_render_pass, None);
        }
    }
}

fn create_render_pass(
    vk_device: &crate::vulkan::VkDeviceHandle,
    swapchain: &crate::vulkan::VkSwapchain,
) -> VkResult<ash::vk::RenderPass> {
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

struct RenderPassDescriptors<Bindings: crate::renderer::Bindings> {
    /// Handle to the vk device
    vk_device: crate::vulkan::VkDeviceHandle,
    /// Descriptor set layout
    descriptor_set_layout: ash::vk::DescriptorSetLayout,
    /// Descriptor pool
    descriptor_pool: ash::vk::DescriptorPool,
    /// Descriptor sets, one per frame in flight
    descriptor_sets: [ash::vk::DescriptorSet; MAX_FRAMES_IN_FLIGHT],

    bindings: Bindings,
}

impl<Bindings: crate::renderer::Bindings> RenderPassDescriptors<Bindings> {
    pub fn create(context: &RenderPassCreateContext) -> Result<Self, NodeCreateError<Bindings>> {
        let device = context.device;

        let desc_set_layout_bindings: Vec<_> = Bindings::desc_set_layout_bindings().collect();
        let descriptor_set_layout_info = ash::vk::DescriptorSetLayoutCreateInfo::default().bindings(&desc_set_layout_bindings);
        let descriptor_set_layout = unsafe { device.create_descriptor_set_layout(&descriptor_set_layout_info, None)? };

        let bindings = Bindings::create(context).map_err(NodeCreateError::BindingsError)?;

        let pool_sizes = [
            ash::vk::DescriptorType::SAMPLER,
            ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            ash::vk::DescriptorType::SAMPLED_IMAGE,
            ash::vk::DescriptorType::STORAGE_IMAGE,
            ash::vk::DescriptorType::UNIFORM_TEXEL_BUFFER,
            ash::vk::DescriptorType::STORAGE_TEXEL_BUFFER,
            ash::vk::DescriptorType::UNIFORM_BUFFER,
            ash::vk::DescriptorType::STORAGE_BUFFER,
            ash::vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
            ash::vk::DescriptorType::STORAGE_BUFFER_DYNAMIC,
            ash::vk::DescriptorType::INPUT_ATTACHMENT,
        ]
        .into_iter()
        .map(|desc_type| (desc_type, Bindings::count_of_type(desc_type) * MAX_FRAMES_IN_FLIGHT as u32))
        .filter(|(_, count)| *count > 0)
        .map(|(desc_type, count)| ash::vk::DescriptorPoolSize::default().ty(desc_type).descriptor_count(count))
        .collect::<Vec<_>>();

        let pool_info = ash::vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(MAX_FRAMES_IN_FLIGHT as u32);

        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        let descriptor_set_layouts = [descriptor_set_layout; MAX_FRAMES_IN_FLIGHT];
        let alloc_info = ash::vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&descriptor_set_layouts);

        let descriptor_sets: [ash::vk::DescriptorSet; MAX_FRAMES_IN_FLIGHT] = unsafe {
            device
                .allocate_descriptor_sets(&alloc_info)?
                .try_into()
                .expect("Descriptor set count mismatch")
        };

        for frame_index in 0..MAX_FRAMES_IN_FLIGHT {
            let descriptor_buffer_infos: Vec<_> = Bindings::descriptor_buffer_infos(&bindings, frame_index)
                .map(|(info, ty)| ([info], ty))
                .collect();
            let writes: Vec<_> = descriptor_buffer_infos
                .iter()
                .enumerate()
                .map(|(binding, (buf_info, desc_type))| {
                    ash::vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_sets[frame_index])
                        .dst_binding(binding as u32)
                        .buffer_info(buf_info)
                        .descriptor_type(*desc_type)
                })
                .collect();

            unsafe { device.update_descriptor_sets(&writes, &[]) }
        }

        Ok(Self {
            vk_device: device.clone(),
            descriptor_set_layout,
            descriptor_pool,
            descriptor_sets,
            bindings,
        })
    }
}

impl<B: Bindings> Drop for RenderPassDescriptors<B> {
    fn drop(&mut self) {
        unsafe {
            self.vk_device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.vk_device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }
}

pub enum NodeCreateError<Bindings: crate::renderer::Bindings> {
    BindingsError(Bindings::CreateError),
    Vulkan(ash::vk::Result),
}

impl<Bindings: crate::renderer::Bindings> From<ash::vk::Result> for NodeCreateError<Bindings> {
    fn from(error: ash::vk::Result) -> Self {
        Self::Vulkan(error)
    }
}

impl<Bindings: crate::renderer::Bindings> std::fmt::Debug for NodeCreateError<Bindings> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BindingsError(err) => write!(f, "NodeCreateError::BindingsError({err})"),
            Self::Vulkan(err) => write!(f, "NodeCreateError::Vulkan({err})"),
        }
    }
}

impl<Bindings: crate::renderer::Bindings> std::fmt::Display for NodeCreateError<Bindings> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BindingsError(err) => write!(f, "Bindings error: {err}"),
            Self::Vulkan(err) => write!(f, "Vulkan error: {err}"),
        }
    }
}

impl<Bindings: crate::renderer::Bindings> std::error::Error for NodeCreateError<Bindings> {}
