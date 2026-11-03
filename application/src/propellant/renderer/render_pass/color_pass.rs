use crate::vulkan;

pub struct ColorPass;

impl super::RenderPass for ColorPass {
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
