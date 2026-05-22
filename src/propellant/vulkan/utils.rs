use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VulkanName([i8; ash::vk::MAX_EXTENSION_NAME_SIZE]);

impl VulkanName {
    pub const fn new(name: &'static str) -> Self {
        assert!(
            name.as_bytes().len() <= ash::vk::MAX_EXTENSION_NAME_SIZE,
            "Unable to create name bigger than 256 bytes"
        );

        let mut result = [0i8; ash::vk::MAX_EXTENSION_NAME_SIZE];

        let bytes = name.as_bytes();
        let mut index = 0;

        while index < bytes.len() {
            result[index] = bytes[index] as i8;
            index += 1;
        }

        Self(result)
    }
}

impl std::ops::Deref for VulkanName {
    type Target = [i8];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
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
