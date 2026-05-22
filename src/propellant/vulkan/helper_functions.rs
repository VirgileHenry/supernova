use crate::propellant::vulkan::VulkanPhysicalProperties;

pub fn find_memory_type<T: std::ops::Deref<Target = ash::Device> + VulkanPhysicalProperties>(
    vulkan_interface: &T,
    type_filter: u32,
    memory_properties: ash::vk::MemoryPropertyFlags,
) -> u32 {
    let properties = vulkan_interface.get_physical_device_memory_properties();
    for i in 0..properties.memory_type_count {
        use std::ops::BitAnd;
        if type_filter & (1 << i) != 0
            && properties.memory_types[i as usize].property_flags.bitand(memory_properties) == memory_properties
        {
            return i;
        }
    }
    panic!("Could not find memory type");
}

// Should make helper func to be reused across all renderer, where should it be ?
pub fn create_image_view(
    vulkan_interface: &impl std::ops::Deref<Target = ash::Device>,
    image: ash::vk::Image,
    format: ash::vk::Format,
) -> ash::vk::ImageView {
    let image_view_create_info = ash::vk::ImageViewCreateInfo {
        image,
        view_type: ash::vk::ImageViewType::TYPE_2D,
        format,
        subresource_range: ash::vk::ImageSubresourceRange {
            aspect_mask: ash::vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
            ..Default::default()
        },

        ..Default::default()
    };
    let view = unsafe { vulkan_interface.create_image_view(&image_view_create_info, None).unwrap() };
    view
}

pub fn create_image<T: std::ops::Deref<Target = ash::Device> + VulkanPhysicalProperties>(
    vulkan_interface: &T,
    width: u32,
    height: u32,
    format: ash::vk::Format,
    tiling: ash::vk::ImageTiling,
    usage: ash::vk::ImageUsageFlags,
    properties: ash::vk::MemoryPropertyFlags,
    mipmapped: bool,
) -> Result<(ash::vk::Image, ash::vk::DeviceMemory), String> {
    let mut image_create_info = ash::vk::ImageCreateInfo {
        image_type: ash::vk::ImageType::TYPE_2D,
        extent: ash::vk::Extent3D { width, height, depth: 1 },
        mip_levels: 1,
        array_layers: 1,
        tiling,
        initial_layout: ash::vk::ImageLayout::UNDEFINED,
        usage,
        samples: ash::vk::SampleCountFlags::TYPE_1,
        sharing_mode: ash::vk::SharingMode::EXCLUSIVE,
        format,
        ..Default::default()
    };

    if mipmapped {
        image_create_info.mip_levels = f32::floor(f32::log2(u32::max(width, height) as f32)) as u32 + 1;
    }

    let image = unsafe { vulkan_interface.create_image(&image_create_info, None).unwrap() };

    let mem_requirements = unsafe { vulkan_interface.get_image_memory_requirements(image) };
    let memory_alloc_info = ash::vk::MemoryAllocateInfo {
        allocation_size: mem_requirements.size,
        memory_type_index: find_memory_type(vulkan_interface, mem_requirements.memory_type_bits, properties),
        ..Default::default()
    };
    let memory = unsafe { vulkan_interface.allocate_memory(&memory_alloc_info, None).unwrap() };

    unsafe {
        vulkan_interface
            .bind_image_memory(image, memory, 0 as ash::vk::DeviceSize)
            .unwrap()
    };

    Ok((image, memory))
}
