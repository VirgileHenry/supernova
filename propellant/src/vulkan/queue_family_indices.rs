/// Index of a queue family.
type QueueFamilyIndex = u32;

#[derive(Debug, Clone, Copy)]
pub struct QueueFamilyIndices {
    /// The family that handles graphics work and presentation.
    pub graphics: QueueFamilyIndex,

    /// Family for async transfers.
    pub transfer: QueueFamilyIndex,

    /// Family for async compute.
    pub compute: QueueFamilyIndex,
}

impl QueueFamilyIndices {
    /// Find the preferred queue family indices for the engine.
    ///
    /// Returns `Ok(None)` if no graphics-capable queue family that can also
    /// present to the given surface exists on this device — i.e. the device
    /// is unsuitable.
    pub fn get(
        instance: &ash::Instance,
        surface_instance: &ash::khr::surface::Instance,
        surface: ash::vk::SurfaceKHR,
        physical_device: ash::vk::PhysicalDevice,
    ) -> ash::prelude::VkResult<Option<QueueFamilyIndices>> {
        let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let graphics = match find_graphics_queue(surface_instance, surface, physical_device, &queue_family_properties)? {
            Some(indices) => indices,
            None => return Ok(None),
        };

        let dedicated_transfer = find_transfer_queue(&queue_family_properties, graphics).unwrap_or(graphics);
        let dedicated_compute = find_compute_queue(&queue_family_properties, graphics).unwrap_or(graphics);

        Ok(Some(QueueFamilyIndices {
            graphics,
            transfer: dedicated_transfer,
            compute: dedicated_compute,
        }))
    }

    pub fn to_queue_create_infos(&self) -> Vec<ash::vk::DeviceQueueCreateInfo<'static>> {
        const QUEUE_PRIORITIES: &[f32] = &[1.0];

        /* Use a hashset to get unique families */
        let mut families = std::collections::HashSet::new();
        families.insert(self.graphics);
        families.insert(self.transfer);
        families.insert(self.compute);

        let mut unique_families: Vec<_> = families.into_iter().collect();
        unique_families.sort();

        unique_families
            .into_iter()
            .map(|family| {
                ash::vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(family)
                    .queue_priorities(QUEUE_PRIORITIES)
            })
            .collect()
    }
}

/// Function to find the most suitable graphics queue.
fn find_graphics_queue(
    surface_instance: &ash::khr::surface::Instance,
    surface: ash::vk::SurfaceKHR,
    physical_device: ash::vk::PhysicalDevice,
    queue_family_properties: &[ash::vk::QueueFamilyProperties],
) -> ash::prelude::VkResult<Option<QueueFamilyIndex>> {
    struct QueueFamilyCandidate<'a> {
        index: u32,
        properties: &'a ash::vk::QueueFamilyProperties,
    }

    let mut graphics_candidates: Vec<QueueFamilyCandidate> = Vec::new();

    for (index, properties) in queue_family_properties.iter().enumerate() {
        let index = index as u32;

        /* Check the graphics flags */
        if !properties.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS) {
            continue;
        }
        /* Check the queue can support our current surface */
        if !unsafe { surface_instance.get_physical_device_surface_support(physical_device, index, surface) }? {
            continue;
        }

        graphics_candidates.push(QueueFamilyCandidate { index, properties });
    }

    /* Get the best candidate. */
    /* The criterion for graphics is the family with the most flags available */
    let best_candidate = graphics_candidates
        .into_iter()
        .max_by_key(|candidate| candidate.properties.queue_flags.as_raw().count_ones())
        .map(|candidate| candidate.index);

    Ok(best_candidate)
}

fn find_transfer_queue(
    queue_family_properties: &[ash::vk::QueueFamilyProperties],
    graphics: QueueFamilyIndex,
) -> Option<QueueFamilyIndex> {
    struct QueueFamilyCandidate<'a> {
        index: u32,
        properties: &'a ash::vk::QueueFamilyProperties,
    }

    let mut transfer_candidates: Vec<QueueFamilyCandidate> = Vec::new();

    for (index, properties) in queue_family_properties.iter().enumerate() {
        let index = index as u32;

        /* Take a different family than the graphics */
        if index == graphics {
            continue;
        }
        /* Check the family has the transfer flag */
        if !properties.queue_flags.contains(ash::vk::QueueFlags::TRANSFER) {
            continue;
        }
        /* Avoid queues with the graphics family, only transfer for dedicated family */
        if properties.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS) {
            continue;
        }

        transfer_candidates.push(QueueFamilyCandidate { index, properties });
    }

    /* Get the best candidate. */
    /* The criterion for transfer is the most specialized queue, i.e. the least feature flags */
    let best_candidate = transfer_candidates
        .into_iter()
        .min_by_key(|candidate| candidate.properties.queue_flags.as_raw().count_ones())
        .map(|candidate| candidate.index);

    best_candidate
}

fn find_compute_queue(
    queue_family_properties: &[ash::vk::QueueFamilyProperties],
    graphics: QueueFamilyIndex,
) -> Option<QueueFamilyIndex> {
    struct QueueFamilyCandidate<'a> {
        index: u32,
        properties: &'a ash::vk::QueueFamilyProperties,
    }

    let mut compute_candidates: Vec<QueueFamilyCandidate> = Vec::new();

    for (index, properties) in queue_family_properties.iter().enumerate() {
        let index = index as u32;

        /* Take a different family than the graphics */
        if index == graphics {
            continue;
        }
        /* Check the family has the compute flag */
        if !properties.queue_flags.contains(ash::vk::QueueFlags::COMPUTE) {
            continue;
        }
        /* Avoid queues with the graphics family, only compute for dedicated family */
        if properties.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS) {
            continue;
        }

        compute_candidates.push(QueueFamilyCandidate { index, properties });
    }

    /* Get the best candidate. */
    /* The criterion for compute is the family with the most queues available */
    let best_candidate = compute_candidates
        .into_iter()
        .max_by_key(|candidate| candidate.properties.queue_count)
        .map(|candidate| candidate.index);

    best_candidate
}
