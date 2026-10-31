pub fn required_device_extensions() -> &'static [&'static std::ffi::CStr] {
    /// Required extensions by the logical device (hence, should be present on the physical device)
    /// This shall match the renderer::vulkan_interface::REQUIRED_DEVICE_EXTENSIONS
    const REQUIRED_DEVICE_EXTENSIONS: &[&std::ffi::CStr] = &[ash::vk::KHR_SWAPCHAIN_NAME, ash::vk::EXT_DESCRIPTOR_INDEXING_NAME];

    REQUIRED_DEVICE_EXTENSIONS
}

pub fn required_device_features() -> ash::vk::PhysicalDeviceFeatures {
    /* Fixme: custom struct and all */
    ash::vk::PhysicalDeviceFeatures::default()
}

/// for all features, if it is in the required, it must be in the physical device
/// In other words, we have the following:
/// required    | actual    | result
/// 0           | 0         | 1
/// 0           | 1         | 1
/// 1           | 0         | 0
/// 1           | 1         | 1
/// so result = !required | actual
/// With u32 instead of bools, the neg is 1-value, and we use bit ops
pub fn physical_device_features_check(
    features: ash::vk::PhysicalDeviceFeatures,
    features_1_2: ash::vk::PhysicalDeviceVulkan12Features,
    features_1_3: ash::vk::PhysicalDeviceVulkan13Features,
) -> bool {
    /* Set all the required features here */
    let required_features = ash::vk::PhysicalDeviceFeatures::default();
    let required_features_1_2 = ash::vk::PhysicalDeviceVulkan12Features::default();
    let required_features_1_3 = ash::vk::PhysicalDeviceVulkan13Features::default();

    (1 - required_features.robust_buffer_access | features.robust_buffer_access)
        & (1 - required_features.full_draw_index_uint32 | features.full_draw_index_uint32)
        & (1 - required_features.image_cube_array | features.image_cube_array)
        & (1 - required_features.independent_blend | features.independent_blend)
        & (1 - required_features.geometry_shader | features.geometry_shader)
        & (1 - required_features.tessellation_shader | features.tessellation_shader)
        & (1 - required_features.sample_rate_shading | features.sample_rate_shading)
        & (1 - required_features.dual_src_blend | features.dual_src_blend)
        & (1 - required_features.logic_op | features.logic_op)
        & (1 - required_features.multi_draw_indirect | features.multi_draw_indirect)
        & (1 - required_features.draw_indirect_first_instance | features.draw_indirect_first_instance)
        & (1 - required_features.depth_clamp | features.depth_clamp)
        & (1 - required_features.depth_bias_clamp | features.depth_bias_clamp)
        & (1 - required_features.fill_mode_non_solid | features.fill_mode_non_solid)
        & (1 - required_features.depth_bounds | features.depth_bounds)
        & (1 - required_features.wide_lines | features.wide_lines)
        & (1 - required_features.large_points | features.large_points)
        & (1 - required_features.alpha_to_one | features.alpha_to_one)
        & (1 - required_features.multi_viewport | features.multi_viewport)
        & (1 - required_features.sampler_anisotropy | features.sampler_anisotropy)
        & (1 - required_features.texture_compression_etc2 | features.texture_compression_etc2)
        & (1 - required_features.texture_compression_astc_ldr | features.texture_compression_astc_ldr)
        & (1 - required_features.texture_compression_bc | features.texture_compression_bc)
        & (1 - required_features.occlusion_query_precise | features.occlusion_query_precise)
        & (1 - required_features.pipeline_statistics_query | features.pipeline_statistics_query)
        & (1 - required_features.vertex_pipeline_stores_and_atomics | features.vertex_pipeline_stores_and_atomics)
        & (1 - required_features.fragment_stores_and_atomics | features.fragment_stores_and_atomics)
        & (1 - required_features.shader_tessellation_and_geometry_point_size
            | features.shader_tessellation_and_geometry_point_size)
        & (1 - required_features.shader_image_gather_extended | features.shader_image_gather_extended)
        & (1 - required_features.shader_storage_image_extended_formats | features.shader_storage_image_extended_formats)
        & (1 - required_features.shader_storage_image_multisample | features.shader_storage_image_multisample)
        & (1 - required_features.shader_storage_image_read_without_format | features.shader_storage_image_read_without_format)
        & (1 - required_features.shader_storage_image_write_without_format | features.shader_storage_image_write_without_format)
        & (1 - required_features.shader_uniform_buffer_array_dynamic_indexing
            | features.shader_uniform_buffer_array_dynamic_indexing)
        & (1 - required_features.shader_sampled_image_array_dynamic_indexing
            | features.shader_sampled_image_array_dynamic_indexing)
        & (1 - required_features.shader_storage_buffer_array_dynamic_indexing
            | features.shader_storage_buffer_array_dynamic_indexing)
        & (1 - required_features.shader_storage_image_array_dynamic_indexing
            | features.shader_storage_image_array_dynamic_indexing)
        & (1 - required_features.shader_clip_distance | features.shader_clip_distance)
        & (1 - required_features.shader_cull_distance | features.shader_cull_distance)
        & (1 - required_features.shader_float64 | features.shader_float64)
        & (1 - required_features.shader_int64 | features.shader_int64)
        & (1 - required_features.shader_int16 | features.shader_int16)
        & (1 - required_features.shader_resource_residency | features.shader_resource_residency)
        & (1 - required_features.shader_resource_min_lod | features.shader_resource_min_lod)
        & (1 - required_features.sparse_binding | features.sparse_binding)
        & (1 - required_features.sparse_residency_buffer | features.sparse_residency_buffer)
        & (1 - required_features.sparse_residency_image2_d | features.sparse_residency_image2_d)
        & (1 - required_features.sparse_residency_image3_d | features.sparse_residency_image3_d)
        & (1 - required_features.sparse_residency2_samples | features.sparse_residency2_samples)
        & (1 - required_features.sparse_residency4_samples | features.sparse_residency4_samples)
        & (1 - required_features.sparse_residency8_samples | features.sparse_residency8_samples)
        & (1 - required_features.sparse_residency16_samples | features.sparse_residency16_samples)
        & (1 - required_features.sparse_residency_aliased | features.sparse_residency_aliased)
        & (1 - required_features.variable_multisample_rate | features.variable_multisample_rate)
        & (1 - required_features.inherited_queries | features.inherited_queries)
        > 0
}

pub fn physical_device_extensions_check(extensions: &std::collections::BTreeSet<String>) -> bool {
    use crate::propellant::vulkan::utils::vulkan_name_to_string;
    let mut required_extensions = required_device_extensions().iter().map(|&required| {
        /* SAFETY: converting &[u8] to &[i8], that's fine */
        let bytes: &[i8] = unsafe { std::mem::transmute(required.to_bytes()) };
        vulkan_name_to_string(bytes)
    });
    required_extensions.all(|required| extensions.contains(&required))
}
