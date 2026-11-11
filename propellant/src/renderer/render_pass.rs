use ash::prelude::VkResult;

pub trait RenderPass {
    const NAME: &'static str;

    type Input;
    type Output;

    type Bindings: crate::renderer::render_graph::Bindings;

    /// Source for the fragment shader, made with the `propellant::frag_shader!` macro.
    const FRAG_SHADER: crate::renderer::shaders::ShaderSource;
    /// Source for the vertex shader, made with the `propellant::vert_shader!` macro.
    const VERT_SHADER: crate::renderer::shaders::ShaderSource;

    fn create() -> Self;
    fn render(
        &mut self,
        context: &RenderPassRenderContext,
        bindings: &Self::Bindings,
        input: Self::Input,
    ) -> VkResult<Self::Output>;
}

pub struct RenderPassCreateContext<'ctx> {
    pub device: &'ctx crate::vulkan::VkDeviceHandle,
    pub swapchain: &'ctx crate::vulkan::VkSwapchain,
    pub assets: &'ctx crate::assets::AssetManager,
}

pub struct RenderPassRenderContext<'ctx> {
    pub device: &'ctx crate::vulkan::VkDeviceHandle,
    pub world: &'ctx hecs::World,
    pub assets: &'ctx crate::assets::AssetManager,
    pub command_buffer: crate::vulkan::InRecordingCommandBuffer,
    pub frame_index: usize,
    pub target: ash::vk::Framebuffer,
    pub render_area: ash::vk::Rect2D,
}
