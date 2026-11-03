use crate::vulkan;

pub struct PostProcessingPass;

impl super::RenderPass for PostProcessingPass {
    type Input<'input> = ();
    type Output<'output> = ();
    fn render<'input, 'output>(
        &mut self,
        vk_device: &vulkan::VkDeviceHandle,
        assets: &crate::assets::AssetManager,
        world: &hecs::World,
        input: &Self::Input<'input>,
        out: &mut Self::Output<'output>,
    ) -> ash::prelude::VkResult<()> {
        Ok(())
    }
}
