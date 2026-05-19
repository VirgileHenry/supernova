use std::ops::Deref;
use crate::vulkan::*;
use crate::vulkan::VulkanPhysicalProperties;

// For now a texture is an image with an image view and a sampler
// We'll see later if we need to split those
// Should impl Texture ?
//Can we use a single sampler for multiple text ?
struct Texture2D {
    image: vk::Image,
    sampler: vk::Sampler,
    view: vk::ImageView,
}

impl Texture2D {

    fn create_sampler<T: Deref<Target=ash::Device> + VulkanPhysicalProperties>(vulkan_interface: &T) -> vk::Sampler {

        let sampler_info = vk::SamplerCreateInfo {
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,

            anisotropy_enable: vk::TRUE,
            max_anisotropy: vulkan_interface
                .get_physical_device_properties()
                .limits
                .max_sampler_anisotropy,

            unnormalized_coordinates: vk::FALSE,
            compare_enable: vk::FALSE,

            mipmap_mode: vk::SamplerMipmapMode::NEAREST,
            mip_lod_bias: 0.,
            min_lod: 0.,
            max_lod: 0.,

            ..Default::default()
        };

        let sampler = unsafe { vulkan_interface.create_sampler(&sampler_info, None).unwrap() };
        sampler

    }

    // Creates a texture
    // For now there are too many args, needs specialization / helpers / presets
    pub fn new(
        vulkan_interface: &VkRendererInterface,
        width: u32,
        height: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Self {
        let (image, memory) = Utils::create_image(
            vulkan_interface,
            width,
            height,
            format,
            tiling,
            usage,
            properties,
            false,
        ).unwrap();
        let view  = Utils::create_image_view(vulkan_interface, image, vk::Format::R8G8B8A8_SRGB);
        let sampler = Self::create_sampler(vulkan_interface);
        Self {
            image,
            view,
            sampler,
        }
    }

}