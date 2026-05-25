use crate::propellant::vulkan;

pub struct ColorPass;

impl super::RenderingPass for ColorPass {
    type Input<'input> = ();
    type Output<'output> = ();
    fn render<'input, 'output>(
        &self,
        world: &hecs::World,
        vk_device: &vulkan::VkDeviceHandle,
        input: &Self::Input<'input>,
        out: &mut Self::Output<'output>,
    ) {
    }
}
