
use crate::vulkan::VkRendererInterface;



pub struct ColorPass;

impl super::RenderingPass for ColorPass {
    type In = ();
    type Out = ();
    fn render(&self, _world: &hecs::World, _vi: &VkRendererInterface, _input: &Self::In, _out: &mut Self::Out) {
        
    }
}
