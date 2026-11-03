use crate::renderer::render_graph::RenderGraphImpl;
use crate::renderer::render_pass::RenderPass;

pub struct RenderGraphNode<Pass: RenderPass> {
    /// Handle to the vk device
    vk_device: crate::vulkan::VkDeviceHandle,
    /// Pipeline layout object
    pipeline_layout: ash::vk::PipelineLayout,
    /// Pipeline object
    pipeline: ash::vk::Pipeline,
    /// Render pass object
    render_pass: ash::vk::RenderPass,

    /// Actual user defined render pass
    pass: Pass,
}

impl<Pass: RenderPass> RenderGraphImpl for RenderGraphNode<Pass> {
    type Input = Pass::Input;
    type Output = Pass::Output;

    type CreateError = NodeCreateError<Pass>;

    fn create(context: crate::renderer::render_graph::RenderPassCreateContext) -> Result<Self, Self::CreateError> {
        let vertex_shader = context.device.load_shader_module(Self::VERTEX_SHADER.code)?;
        let fragment_shader = context.device.load_shader_module(Self::FRAGMENT_SHADER.code)?;

        let vert_stage = ash::vk::PipelineShaderStageCreateInfo::default()
            .stage(ash::vk::ShaderStageFlags::VERTEX)
            .module(vertex_shader)
            .name(Self::VERTEX_SHADER.entry_point);

        let frag_stage = ash::vk::PipelineShaderStageCreateInfo::default()
            .stage(ash::vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_shader)
            .name(Self::FRAGMENT_SHADER.entry_point);

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
    }

    fn render(
        &mut self,
        vk_device: &crate::vulkan::VkDeviceHandle,
        assets: &crate::assets::AssetManager,
        world: &hecs::World,
        input: Self::Input,
    ) -> ash::prelude::VkResult<Self::Output> {
        /* Todo: bind uniforms */
        self.pass.render(vk_device, assets, world, input)
    }
}

#[derive(Debug)]
pub enum NodeCreateError<Pass: RenderPass> {}

impl<Pass: RenderPass> std::fmt::Display for NodeCreateError<Pass> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {}
}

impl<Pass: RenderPass> std::error::Error for NodeCreateError<Pass> {}
