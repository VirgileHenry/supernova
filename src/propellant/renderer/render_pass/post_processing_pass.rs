use crate::propellant::vulkan;

pub struct PostProcessingPass;

impl super::RenderingPass for PostProcessingPass {
    type In = ();
    type Out = ();
    fn render(&self, _world: &hecs::World, _vi: &vulkan::VkRendererInterface, _input: &Self::In, _out: &mut Self::Out) {}
}
