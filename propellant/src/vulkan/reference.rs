/// Zero cost wrapper type to indicate that we are not owning the underlying vulkan resource.
///
/// Hence, you should not destroy it!
#[derive(Debug, Clone)]
pub struct VkRef<T>(T);

impl<T> std::ops::Deref for VkRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone> VkRef<T> {
    pub fn shared(t: &T) -> VkRef<T> {
        VkRef(t.clone())
    }
}
