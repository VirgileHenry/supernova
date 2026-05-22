use crate::propellant::vulkan;

// For now a texture is an image with an image view and a sampler
// We'll see later if we need to split those
// Should impl Texture ?
//Can we use a single sampler for multiple text ?
struct Texture2D {
    image: ash::vk::Image,
    sampler: ash::vk::Sampler,
    view: ash::vk::ImageView,
}

impl Texture2D {
    fn create_sampler<T: std::ops::Deref<Target = ash::Device> + vulkan::VulkanPhysicalProperties>(
        vulkan_interface: &T,
    ) -> ash::vk::Sampler {
        let sampler_info = ash::vk::SamplerCreateInfo {
            mag_filter: ash::vk::Filter::LINEAR,
            min_filter: ash::vk::Filter::LINEAR,
            address_mode_u: ash::vk::SamplerAddressMode::REPEAT,
            address_mode_v: ash::vk::SamplerAddressMode::REPEAT,
            address_mode_w: ash::vk::SamplerAddressMode::REPEAT,
            border_color: ash::vk::BorderColor::INT_OPAQUE_BLACK,

            anisotropy_enable: ash::vk::TRUE,
            max_anisotropy: vulkan_interface
                .get_physical_device_properties()
                .limits
                .max_sampler_anisotropy,

            unnormalized_coordinates: ash::vk::FALSE,
            compare_enable: ash::vk::FALSE,

            mipmap_mode: ash::vk::SamplerMipmapMode::NEAREST,
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
        vulkan_interface: &vulkan::VkRendererInterface,
        width: u32,
        height: u32,
        format: ash::vk::Format,
        tiling: ash::vk::ImageTiling,
        usage: ash::vk::ImageUsageFlags,
        properties: ash::vk::MemoryPropertyFlags,
    ) -> Self {
        let (image, memory) =
            vulkan::create_image(vulkan_interface, width, height, format, tiling, usage, properties, false).unwrap();
        let view = vulkan::create_image_view(vulkan_interface, image, ash::vk::Format::R8G8B8A8_SRGB);
        let sampler = Self::create_sampler(vulkan_interface);
        Self { image, view, sampler }
    }
}
