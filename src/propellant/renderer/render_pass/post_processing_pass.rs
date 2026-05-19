

use crate::vulkan::VkRendererInterface;


pub struct PostProcessingPass;

impl super::RenderingPass for PostProcessingPass {
    type In = ();
    type Out = ();
    fn render(&self, _world: &hecs::World, _vi: &VkRendererInterface, _input: &Self::In, _out: &mut Self::Out) {
        
    }
}


