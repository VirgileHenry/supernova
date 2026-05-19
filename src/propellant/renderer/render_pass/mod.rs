
use crate::vulkan::VkRendererInterface;



pub mod geometry_pass;
pub mod color_pass;
pub mod post_processing_pass;


pub trait RenderingPass {
    type In;
    type Out;
    fn render(&self, world: &hecs::World, vi: &VkRendererInterface, input: &Self::In, out: &mut Self::Out);
}

#[derive(Debug, Clone, Copy)]
pub struct RenderPassTarget {
    pub framebuffer: ash::vk::Framebuffer,
}

impl RenderPassTarget {
    pub fn create(
        device: &ash::Device,
        render_pass: ash::vk::RenderPass,
        attachments: &[ash::vk::ImageView],
        extent: ash::vk::Extent2D
    ) -> Result<RenderPassTarget, crate::ScError> {
        let framebuffer_create_info = ash::vk::FramebufferCreateInfo {
            render_pass,
            attachment_count: attachments.len() as u32,
            p_attachments: if attachments.is_empty() { std::ptr::null() } else { attachments.as_ptr() },
            width: extent.width,
            height: extent.height,
            layers: 1, // TODO
            ..Default::default()
        };
        
        let framebuffer = unsafe { device.create_framebuffer(&framebuffer_create_info, None)? };

        Ok(RenderPassTarget {
            framebuffer,
            // semaphore,
        })
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_framebuffer(self.framebuffer, None);
        }
    }
}



