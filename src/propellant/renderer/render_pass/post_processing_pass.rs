use crate::propellant::vulkan;

pub struct PostProcessingPass;

impl super::RenderingPass for PostProcessingPass {
    type Input<'input> = ();
    type Output<'output> = ();
    fn render<'input, 'output>(
        &self,
        vk_device: &vulkan::VkDeviceHandle,
        assets: &crate::propellant::assets::AssetManager,
        world: &hecs::World,
        input: &Self::Input<'input>,
        out: &mut Self::Output<'output>,
    ) -> ash::prelude::VkResult<()> {
        Ok(())
    }
}
