use crate::propellant::vulkan;

pub struct ColorPass;

impl super::RenderingPass for ColorPass {
    type In = ();
    type Out = ();
    fn render(&self, _world: &hecs::World, _vi: &vulkan::VkRendererInterface, _input: &Self::In, _out: &mut Self::Out) {}
}
