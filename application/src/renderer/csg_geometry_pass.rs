use propellant::renderer::shaders::ShaderSource;

pub struct CsgGeometryPass {}

impl propellant::renderer::RenderPass for CsgGeometryPass {
    type Input = propellant::renderer::RenderGraphInput;
    type Output = propellant::renderer::RenderGraphOutput;

    const FRAG_SHADER: ShaderSource = propellant::frag_shader!("shaders/segment.frag.glsl");
    const VERT_SHADER: ShaderSource = propellant::vert_shader!("shaders/segment.vert.glsl");

    fn render(
        &mut self,
        vk_device: &propellant::vulkan::VkDeviceHandle,
        assets: &propellant::assets::AssetManager,
        world: &hecs::World,
        input: Self::Input,
    ) -> ash::prelude::VkResult<Self::Output> {
    }
}
