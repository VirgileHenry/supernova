/// Wrapper around an image swapchain.
pub struct VkSwapchainImage {
    _image: ash::vk::Image,
    view: ash::vk::ImageView,
}

impl VkSwapchainImage {
    pub fn create(
        vk_device: &crate::propellant::vulkan::VkDeviceHandle,
        image: ash::vk::Image,
        format: ash::vk::Format,
    ) -> Result<VkSwapchainImage, crate::ScError> {
        let components = ash::vk::ComponentMapping {
            r: ash::vk::ComponentSwizzle::IDENTITY,
            g: ash::vk::ComponentSwizzle::IDENTITY,
            b: ash::vk::ComponentSwizzle::IDENTITY,
            a: ash::vk::ComponentSwizzle::IDENTITY,
        };
        let subresource_range = ash::vk::ImageSubresourceRange {
            aspect_mask: ash::vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };
        let image_view_create_info = ash::vk::ImageViewCreateInfo {
            image,
            view_type: ash::vk::ImageViewType::TYPE_2D,
            format,
            components,
            subresource_range,
            ..Default::default()
        };

        let view = unsafe { vk_device.create_image_view(&image_view_create_info, None)? };
        Ok(VkSwapchainImage { _image: image, view })
    }

    pub fn view(&self) -> ash::vk::ImageView {
        self.view
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_image_view(self.view, None);
        }
    }
}
