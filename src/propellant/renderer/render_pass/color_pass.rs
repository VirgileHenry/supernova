use crate::propellant::vulkan;

pub struct ColorPass;

impl super::RenderingPass for ColorPass {
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
