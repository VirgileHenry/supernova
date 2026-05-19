use std::ops::Deref;



pub type VulkanName = [i8; ash::vk::MAX_EXTENSION_NAME_SIZE];

pub const fn create_vulkan_name(name: &'static str) -> VulkanName {
    let mut result = [0i8; ash::vk::MAX_EXTENSION_NAME_SIZE];

    if name.as_bytes().len() > ash::vk::MAX_EXTENSION_NAME_SIZE {
        panic!("Unable to create name bigger than 256 bytes")
    }

    let bytes = name.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        result[index] = bytes[index] as i8;
        index += 1;
    }

    result
}

/// Zero cost wrapper type to indicate that we are not owning the underlying vulkan resource.
/// 
/// Hence, you should not destroy it!
pub struct VkShared<T>(T);

impl<T> std::ops::Deref for VkShared<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for VkShared<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Clone> Clone for VkShared<T> {
    fn clone(&self) -> Self {
        VkShared(self.deref().clone())
    }
}

impl<T: Clone> VkShared<T> {
    pub fn shared(t: &T) -> VkShared<T> {
        VkShared(t.clone())
    }
}
