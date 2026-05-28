use crate::propellant::vulkan;

pub mod color_pass;
pub mod geometry_pass;
pub mod post_processing_pass;

pub trait RenderingPass {
    type Input<'input>;
    type Output<'output>;
    fn render<'input, 'output>(
        &self,
        vk_device: &vulkan::VkDeviceHandle,
        assets: &crate::propellant::assets::AssetManager,
        world: &hecs::World,
        input: &Self::Input<'input>,
        out: &mut Self::Output<'output>,
    ) -> ash::prelude::VkResult<()>;
}

#[derive(Debug, Clone)]
pub struct RenderPassTarget {
    /// Handle to the vulkan device to use the vulkan API.
    vk_device: vulkan::VkDeviceHandle,
    /// Target framebuffer to write to.
    pub framebuffer: ash::vk::Framebuffer,
}

impl RenderPassTarget {
    pub fn create(
        vk_device: &crate::propellant::vulkan::VkDeviceHandle,
        render_pass: ash::vk::RenderPass,
        attachments: &[ash::vk::ImageView],
        extent: ash::vk::Extent2D,
    ) -> Result<RenderPassTarget, crate::ScError> {
        let framebuffer_create_info = ash::vk::FramebufferCreateInfo {
            render_pass,
            attachment_count: attachments.len() as u32,
            p_attachments: if attachments.is_empty() {
                std::ptr::null()
            } else {
                attachments.as_ptr()
            },
            width: extent.width,
            height: extent.height,
            layers: 1, // TODO
            ..Default::default()
        };

        let framebuffer = unsafe { vk_device.create_framebuffer(&framebuffer_create_info, None)? };

        Ok(RenderPassTarget {
            vk_device: vk_device.clone(),
            framebuffer,
        })
    }
}

impl Drop for RenderPassTarget {
    fn drop(&mut self) {
        unsafe { self.vk_device.destroy_framebuffer(self.framebuffer, None) }
    }
}
