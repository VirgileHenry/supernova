#[derive(Debug, Clone, Copy)]
pub struct QueueFamilyIndices {
    /// Index of the mainstream graphics queue family
    pub graphics: (u32, u32),
    /// Index of the presentation queue family
    /// best if the same as graphics, to avoid concurrent access to images through queues ?
    pub present: Option<(u32, u32)>,
    /// Index of the transfer dedicated queue family index.
    /// If this is none, the GPU only allows to build one queue.
    /// Otherwise, we have the index and offset of the queue.
    /// If the index is different than the graphics queue, offset would be zero
    /// If offset is one, we are using the next queue of the graphics queue family.
    pub transfer: Option<(u32, u32)>,
    /// Index of any dedicated compute queue.
    /// Same logic as for the transfer queue ?
    pub compute: Option<(u32, u32)>,
}

impl QueueFamilyIndices {
    /// Get the different prefered queue family indices.
    /// TODO: This whole things is to redo, taking into account:
    /// - Maybe compute and transfer index ca be the same as graphics, if enough queues can be built
    /// - I have no clue how to pick the best queue if we have multiple choices
    pub fn get(
        instance: &ash::Instance,
        surface_instance: &ash::khr::surface::Instance,
        physical_device: ash::vk::PhysicalDevice,
        surface: ash::vk::SurfaceKHR,
    ) -> Result<QueueFamilyIndices, crate::ScError> {
        let queues_family_indices = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let mut used_indices = std::collections::HashMap::new();

        // create a special closure to validate present queue: we need to fetch the fact that the queue can present from an surface instance call
        // we use a closure so it can capture the required instance & surface, without bloating the required closure params for every other queue
        let validate_present_index = create_validate_present_index(surface_instance, physical_device, surface);

        let graphics = Self::get_best_index_for(
            &queues_family_indices,
            validate_graphics_index,
            score_graphics_index,
            &mut used_indices,
        )
        .ok_or("No Graphics queue family available!")?;
        let present = Self::get_best_index_for(
            &queues_family_indices,
            validate_present_index,
            score_present_index,
            &mut used_indices,
        );
        let transfer = Self::get_best_index_for(
            &queues_family_indices,
            validate_transfer_index,
            score_transfer_index,
            &mut used_indices,
        );
        let compute = Self::get_best_index_for(
            &queues_family_indices,
            validate_compute_index,
            score_compute_index,
            &mut used_indices,
        );

        Ok(QueueFamilyIndices {
            graphics,
            present,
            transfer,
            compute,
        })
    }

    fn get_best_index_for<ValidateFunc, ScoreFunc>(
        queues_family_indices: &[ash::vk::QueueFamilyProperties],
        v_func: ValidateFunc,
        s_func: ScoreFunc,
        used_indices: &mut std::collections::HashMap<usize, u32>,
    ) -> Option<(u32, u32)>
    where
        ValidateFunc: Fn(usize, ash::vk::QueueFamilyProperties) -> bool,
        ScoreFunc: Fn(ash::vk::QueueFamilyProperties) -> i32,
    {
        // Get the index of the queue family that best fit our validate and score functions, while still being available
        let chosen_index = queues_family_indices
            .iter()
            .enumerate()
            // Keep only family properties that match the validate function
            .filter(|(index, properties)| v_func(*index, **properties))
            // Keep only families with remaining queues
            .filter(|(i, properties)| match used_indices.get(i) {
                Some(used_count) => properties.queue_count > *used_count,
                None => properties.queue_count > 0,
            })
            // Get the family with best score
            .max_by(|(_, prop1), (_, prop2)| s_func(**prop1).cmp(&s_func(**prop2)))
            .map(|(index, _)| index)?;

        // update the used index with our pick and get our offset
        let offset = match used_indices.get_mut(&chosen_index) {
            Some(used_count) => {
                let offset = *used_count;
                *used_count += 1;
                offset
            }
            None => {
                used_indices.insert(chosen_index, 1);
                0
            }
        };
        Some((chosen_index as u32, offset))
    }
}

fn validate_graphics_index(_: usize, properties: ash::vk::QueueFamilyProperties) -> bool {
    properties.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS)
}

fn score_graphics_index(_: ash::vk::QueueFamilyProperties) -> i32 {
    0 // TODO
}

fn create_validate_present_index(
    instance: &ash::khr::surface::Instance,
    physical_device: ash::vk::PhysicalDevice,
    surface: ash::vk::SurfaceKHR,
) -> impl Fn(usize, ash::vk::QueueFamilyProperties) -> bool + '_ {
    move |queue_family_index, _| match unsafe {
        instance.get_physical_device_surface_support(physical_device, queue_family_index as u32, surface)
    } {
        Ok(res) => res,
        Err(e) => {
            log::warn!("Failed to read device surface support: {e}, default to false");
            false
        }
    }
}

fn score_present_index(_: ash::vk::QueueFamilyProperties) -> i32 {
    0 // TODO
}

fn validate_transfer_index(_: usize, properties: ash::vk::QueueFamilyProperties) -> bool {
    properties.queue_flags.contains(ash::vk::QueueFlags::TRANSFER)
}

fn score_transfer_index(properties: ash::vk::QueueFamilyProperties) -> i32 {
    // the least flags on the queue, the better: it means the queue is desiged for transfer
    let mut score = 0;
    score -= properties.queue_flags.as_raw().count_ones() as i32;
    score
}

fn validate_compute_index(_: usize, properties: ash::vk::QueueFamilyProperties) -> bool {
    properties.queue_flags.contains(ash::vk::QueueFlags::COMPUTE)
}

fn score_compute_index(properties: ash::vk::QueueFamilyProperties) -> i32 {
    // the least flags on the queue, the better: it means the queue is desiged for transfer
    let mut score = 0;
    score -= properties.queue_flags.as_raw().count_ones() as i32;
    score
}
