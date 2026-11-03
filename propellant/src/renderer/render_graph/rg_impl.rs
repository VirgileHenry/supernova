pub trait RenderGraphImpl {
    type Input: Copy;
    type Output: Copy;
    type CreateError: std::error::Error;

    fn create(context: crate::renderer::render_graph::RenderPassCreateContext) -> Result<Self, Self::CreateError>;

    fn render(
        &mut self,
        vk_device: &crate::vulkan::VkDeviceHandle,
        assets: &crate::assets::AssetManager,
        world: &hecs::World,
        input: Self::Input,
    ) -> ash::prelude::VkResult<Self::Output>;
}
