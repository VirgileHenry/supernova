/// Wrapper around a Vulkan Queue Object.
#[derive(Debug, Clone, Copy)]
pub struct VkQueue {
    handle: ash::vk::Queue,
    family: u32,
}

impl VkQueue {
    pub fn create(device: &ash::Device, queue_family_index: u32) -> Self {
        Self {
            handle: unsafe { device.get_device_queue(queue_family_index, 0) },
            family: queue_family_index,
        }
    }

    pub fn family(&self) -> u32 {
        self.family
    }
}

impl std::ops::Deref for VkQueue {
    type Target = ash::vk::Queue;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

/// A vulkan queue that is either owned or shared.
///
/// If the variant is Owned, the current queue is ours and it is
/// our own responsability to destroy it.
///
/// If the variant is shared, we can't destroy it.
#[derive(Debug, Clone)]
pub enum VkSharableQueue {
    Owned(VkQueue),
    Shared(crate::propellant::vulkan::VkRef<VkQueue>),
}

impl VkSharableQueue {
    pub fn ownership(&self) -> &'static str {
        match self {
            Self::Owned(_) => "Owned",
            Self::Shared(_) => "Shared",
        }
    }

    pub fn as_ref(&self) -> &VkQueue {
        match self {
            Self::Owned(queue) => queue,
            Self::Shared(queue) => queue,
        }
    }
}
