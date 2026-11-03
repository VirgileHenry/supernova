use crate::vulkan;

pub mod color_pass;
pub mod geometry_pass;
pub mod post_processing_pass;

pub trait RenderPass {
    type Input: Copy;
    type Output: Copy;

    /// Source for the fragment shader, made with the `propellant::frag_shader!` macro.
    const FRAG_SHADER: crate::renderer::shaders::ShaderSource;
    /// Source for the vertex shader, made with the `propellant::vert_shader!` macro.
    const VERT_SHADER: crate::renderer::shaders::ShaderSource;

    fn render(
        &mut self,
        vk_device: &vulkan::VkDeviceHandle,
        assets: &crate::assets::AssetManager,
        world: &hecs::World,
        input: Self::Input,
    ) -> ash::prelude::VkResult<Self::Output>;
}
