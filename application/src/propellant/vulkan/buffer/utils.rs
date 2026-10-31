/// Find a memory type index that satisfies both the buffer's type bits
/// and the requested property flags.
///
/// Returns None if no such memory type exists on this device.
pub fn find_memory_type(
    physical_device: &crate::propellant::vulkan::VkPhysicalDevice,
    type_bits: u32,
    required_properties: ash::vk::MemoryPropertyFlags,
) -> Option<u32> {
    let mem_props = physical_device.memory_properties();
    (0..mem_props.memory_type_count).find(|&i| {
        let type_supported = (type_bits & (1 << i)) != 0;
        let props_supported = mem_props.memory_types[i as usize]
            .property_flags
            .contains(required_properties);
        type_supported && props_supported
    })
}
