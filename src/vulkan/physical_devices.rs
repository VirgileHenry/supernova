
/// Required features to use the physical device.
/// If at any point we need a feature, set it to 1 in this list.
const REQUIRED_DEVICE_FEATURES: ash::vk::PhysicalDeviceFeatures = ash::vk::PhysicalDeviceFeatures {
    robust_buffer_access: ash::vk::FALSE,
    full_draw_index_uint32: ash::vk::FALSE,
    image_cube_array: ash::vk::FALSE,
    independent_blend: ash::vk::FALSE,
    geometry_shader: ash::vk::FALSE,
    tessellation_shader: ash::vk::FALSE,
    sample_rate_shading: ash::vk::FALSE,
    dual_src_blend: ash::vk::FALSE,
    logic_op: ash::vk::FALSE,
    multi_draw_indirect: ash::vk::FALSE,
    draw_indirect_first_instance: ash::vk::FALSE,
    depth_clamp: ash::vk::FALSE,
    depth_bias_clamp: ash::vk::FALSE,
    fill_mode_non_solid: ash::vk::FALSE,
    depth_bounds: ash::vk::FALSE,
    wide_lines: ash::vk::FALSE,
    large_points: ash::vk::FALSE,
    alpha_to_one: ash::vk::FALSE,
    multi_viewport: ash::vk::FALSE,
    sampler_anisotropy: ash::vk::FALSE,
    texture_compression_etc2: ash::vk::FALSE,
    texture_compression_astc_ldr: ash::vk::FALSE,
    texture_compression_bc: ash::vk::FALSE,
    occlusion_query_precise: ash::vk::FALSE,
    pipeline_statistics_query: ash::vk::FALSE,
    vertex_pipeline_stores_and_atomics: ash::vk::FALSE,
    fragment_stores_and_atomics: ash::vk::FALSE,
    shader_tessellation_and_geometry_point_size: ash::vk::FALSE,
    shader_image_gather_extended: ash::vk::FALSE,
    shader_storage_image_extended_formats: ash::vk::FALSE,
    shader_storage_image_multisample: ash::vk::FALSE,
    shader_storage_image_read_without_format: ash::vk::FALSE,
    shader_storage_image_write_without_format: ash::vk::FALSE,
    shader_uniform_buffer_array_dynamic_indexing: ash::vk::FALSE,
    shader_sampled_image_array_dynamic_indexing: ash::vk::FALSE,
    shader_storage_buffer_array_dynamic_indexing: ash::vk::FALSE,
    shader_storage_image_array_dynamic_indexing: ash::vk::FALSE,
    shader_clip_distance: ash::vk::FALSE,
    shader_cull_distance: ash::vk::FALSE,
    shader_float64: ash::vk::FALSE,
    shader_int64: ash::vk::FALSE,
    shader_int16: ash::vk::FALSE,
    shader_resource_residency: ash::vk::FALSE,
    shader_resource_min_lod: ash::vk::FALSE,
    sparse_binding: ash::vk::FALSE,
    sparse_residency_buffer: ash::vk::FALSE,
    sparse_residency_image2_d: ash::vk::FALSE,
    sparse_residency_image3_d: ash::vk::FALSE,
    sparse_residency2_samples: ash::vk::FALSE,
    sparse_residency4_samples: ash::vk::FALSE,
    sparse_residency8_samples: ash::vk::FALSE,
    sparse_residency16_samples: ash::vk::FALSE,
    sparse_residency_aliased: ash::vk::FALSE,
    variable_multisample_rate: ash::vk::FALSE,
    inherited_queries: ash::vk::FALSE,
};

/// Required extensions by the logical device (hence, should be present on the physical device)
/// This shall match the renderer::vulkan_interface::REQUIRED_DEVICE_EXTENSIONS
const REQUIRED_DEVICE_EXTENSIONS: &[crate::vulkan::utils::VulkanName] = &[
    crate::vulkan::utils::create_vulkan_name("VK_KHR_swapchain"),
    crate::vulkan::utils::create_vulkan_name("VK_EXT_descriptor_indexing"),
];


pub fn valid_physical_devices<'a>(instance: &'a ash::Instance, surface_instance: &'a ash::khr::surface::Instance, surface: ash::vk::SurfaceKHR) -> Option<impl Iterator<Item = ash::vk::PhysicalDevice> + 'a> {
    Some(
        unsafe { instance.enumerate_physical_devices() }.ok()?
            .into_iter()
            .filter(move |physical_device| validate_physical_device(instance, surface_instance, *physical_device, surface))
    )
}

pub fn best_physical_device(instance: &ash::Instance, surface_instance: &ash::khr::surface::Instance, surface: ash::vk::SurfaceKHR) -> Option<(ash::vk::PhysicalDevice, super::queue_family_indices::QueueFamilyIndices)> {
    valid_physical_devices(instance, surface_instance, surface)?
        .map(|device| {
            let queue_family_indices = super::queue_family_indices::QueueFamilyIndices::get(instance, surface_instance, device, surface);
            (device, queue_family_indices) 
        })
        .filter(|(_, queue_family_indices)| queue_family_indices.is_ok())
        .map(|(device, queue_family_indices)| (device, queue_family_indices.unwrap()))
        .max_by(|(dev1, qfi1), (dev2, qfi2)| {
            score_physical_device(*dev1, *qfi1).cmp(&score_physical_device(*dev2, *qfi2))
        })
}

/// Attempts to score the physical devices to pick the best one.
/// This function shall take in more parameters and be improved.
fn score_physical_device(
    _physical_device: ash::vk::PhysicalDevice,
    queue_family_indices: super::queue_family_indices::QueueFamilyIndices
) -> i32 {
    let mut score = 0;
    
    if queue_family_indices.transfer.is_some() {
        score += 1; // nicer to have separate transfer queue
    }
    if queue_family_indices.compute.is_some() {
        score += 1; // nicer to have dedicated compute queue
    }
    
    score
}

/// Tells whether a physical device meets the mininmum requirements to be used in our app.
/// The requirements can be features, extensions, properties, etc.
fn validate_physical_device(instance: &ash::Instance, surface_instance: &ash::khr::surface::Instance, physical_device: ash::vk::PhysicalDevice, surface: ash::vk::SurfaceKHR) -> bool {
    // physical device should have at least one queue that can perform present ops
    let at_least_one_queue_can_present = unsafe { instance.get_physical_device_queue_family_properties(physical_device) }.iter()
        .enumerate()
        .map(|(queue_family_index, _)| unsafe { surface_instance.get_physical_device_surface_support(physical_device, queue_family_index as u32, surface) })
        .any(|res| res == Ok(true));
    // check for physical device features
    let physical_device_contains_all_features = physical_device_contains_all_features(unsafe { instance.get_physical_device_features(physical_device) });
    // TODO: also check there is enough queues for everything ? so if compute queue is none, we can fallback on graphics queue but can it also do compute ?

    // check for physical device extensions
    let check_physical_device_extensions = match check_physical_device_extensions(instance, physical_device) {
        Ok(res) => res,
        Err(e) => {
            log::warn!("Failed to check extensions for physical device {physical_device:?}: {e}, default to false");
            false
        }
    };

    // early check as we need swapchain extension before looking its support
    if !check_physical_device_extensions {
        return false;
    }

    // check the device can create the swapchain
    let device_has_sapchain_support = match super::swapchain_support::SwapchainSupport::get(surface_instance, physical_device, surface) {
        Ok(swapchain_support) => swapchain_support.suitable(),
        Err(e) => {
            log::warn!("Failed to fetch swapchain support for physical device {physical_device:?}: {e}, default to false");
            false
        }
    };
    
    at_least_one_queue_can_present && physical_device_contains_all_features && device_has_sapchain_support
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
fn physical_device_contains_all_features(features: ash::vk::PhysicalDeviceFeatures) -> bool {
    (1 - REQUIRED_DEVICE_FEATURES.robust_buffer_access | features.robust_buffer_access) &
    (1 - REQUIRED_DEVICE_FEATURES.full_draw_index_uint32 | features.full_draw_index_uint32) &
    (1 - REQUIRED_DEVICE_FEATURES.image_cube_array | features.image_cube_array) &
    (1 - REQUIRED_DEVICE_FEATURES.independent_blend | features.independent_blend) &
    (1 - REQUIRED_DEVICE_FEATURES.geometry_shader | features.geometry_shader) &
    (1 - REQUIRED_DEVICE_FEATURES.tessellation_shader | features.tessellation_shader) &
    (1 - REQUIRED_DEVICE_FEATURES.sample_rate_shading | features.sample_rate_shading) &
    (1 - REQUIRED_DEVICE_FEATURES.dual_src_blend | features.dual_src_blend) &
    (1 - REQUIRED_DEVICE_FEATURES.logic_op | features.logic_op) &
    (1 - REQUIRED_DEVICE_FEATURES.multi_draw_indirect | features.multi_draw_indirect) &
    (1 - REQUIRED_DEVICE_FEATURES.draw_indirect_first_instance | features.draw_indirect_first_instance) &
    (1 - REQUIRED_DEVICE_FEATURES.depth_clamp | features.depth_clamp) &
    (1 - REQUIRED_DEVICE_FEATURES.depth_bias_clamp | features.depth_bias_clamp) &
    (1 - REQUIRED_DEVICE_FEATURES.fill_mode_non_solid | features.fill_mode_non_solid) &
    (1 - REQUIRED_DEVICE_FEATURES.depth_bounds | features.depth_bounds) &
    (1 - REQUIRED_DEVICE_FEATURES.wide_lines | features.wide_lines) &
    (1 - REQUIRED_DEVICE_FEATURES.large_points | features.large_points) &
    (1 - REQUIRED_DEVICE_FEATURES.alpha_to_one | features.alpha_to_one) &
    (1 - REQUIRED_DEVICE_FEATURES.multi_viewport | features.multi_viewport) &
    (1 - REQUIRED_DEVICE_FEATURES.sampler_anisotropy | features.sampler_anisotropy) &
    (1 - REQUIRED_DEVICE_FEATURES.texture_compression_etc2 | features.texture_compression_etc2) &
    (1 - REQUIRED_DEVICE_FEATURES.texture_compression_astc_ldr | features.texture_compression_astc_ldr) &
    (1 - REQUIRED_DEVICE_FEATURES.texture_compression_bc | features.texture_compression_bc) &
    (1 - REQUIRED_DEVICE_FEATURES.occlusion_query_precise | features.occlusion_query_precise) &
    (1 - REQUIRED_DEVICE_FEATURES.pipeline_statistics_query | features.pipeline_statistics_query) &
    (1 - REQUIRED_DEVICE_FEATURES.vertex_pipeline_stores_and_atomics | features.vertex_pipeline_stores_and_atomics) &
    (1 - REQUIRED_DEVICE_FEATURES.fragment_stores_and_atomics | features.fragment_stores_and_atomics) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_tessellation_and_geometry_point_size | features.shader_tessellation_and_geometry_point_size) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_image_gather_extended | features.shader_image_gather_extended) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_storage_image_extended_formats | features.shader_storage_image_extended_formats) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_storage_image_multisample | features.shader_storage_image_multisample) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_storage_image_read_without_format | features.shader_storage_image_read_without_format) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_storage_image_write_without_format | features.shader_storage_image_write_without_format) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_uniform_buffer_array_dynamic_indexing | features.shader_uniform_buffer_array_dynamic_indexing) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_sampled_image_array_dynamic_indexing | features.shader_sampled_image_array_dynamic_indexing) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_storage_buffer_array_dynamic_indexing | features.shader_storage_buffer_array_dynamic_indexing) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_storage_image_array_dynamic_indexing | features.shader_storage_image_array_dynamic_indexing) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_clip_distance | features.shader_clip_distance) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_cull_distance | features.shader_cull_distance) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_float64 | features.shader_float64) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_int64 | features.shader_int64) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_int16 | features.shader_int16) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_resource_residency | features.shader_resource_residency) &
    (1 - REQUIRED_DEVICE_FEATURES.shader_resource_min_lod | features.shader_resource_min_lod) &
    (1 - REQUIRED_DEVICE_FEATURES.sparse_binding | features.sparse_binding) &
    (1 - REQUIRED_DEVICE_FEATURES.sparse_residency_buffer | features.sparse_residency_buffer) &
    (1 - REQUIRED_DEVICE_FEATURES.sparse_residency_image2_d | features.sparse_residency_image2_d) &
    (1 - REQUIRED_DEVICE_FEATURES.sparse_residency_image3_d | features.sparse_residency_image3_d) &
    (1 - REQUIRED_DEVICE_FEATURES.sparse_residency2_samples | features.sparse_residency2_samples) &
    (1 - REQUIRED_DEVICE_FEATURES.sparse_residency4_samples | features.sparse_residency4_samples) &
    (1 - REQUIRED_DEVICE_FEATURES.sparse_residency8_samples | features.sparse_residency8_samples) &
    (1 - REQUIRED_DEVICE_FEATURES.sparse_residency16_samples | features.sparse_residency16_samples) &
    (1 - REQUIRED_DEVICE_FEATURES.sparse_residency_aliased | features.sparse_residency_aliased) &
    (1 - REQUIRED_DEVICE_FEATURES.variable_multisample_rate | features.variable_multisample_rate) &
    (1 - REQUIRED_DEVICE_FEATURES.inherited_queries | features.inherited_queries) > 0
}


fn check_physical_device_extensions(instance: &ash::Instance, physical_device: ash::vk::PhysicalDevice) -> Result<bool, crate::ScError> {
    let extensions = unsafe { instance.enumerate_device_extension_properties(physical_device)? }.iter()
        .map(|extension| extension.extension_name)
        .collect::<std::collections::HashSet<_>>();
    
    for required_extension in REQUIRED_DEVICE_EXTENSIONS {
        if !extensions.contains(required_extension) {
            return Ok(false);
        }
    }

    Ok(true)

}


