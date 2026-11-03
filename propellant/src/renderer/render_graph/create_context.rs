pub struct RenderPassCreateContext<'ctx> {
    pub device: &'ctx crate::vulkan::VkDeviceHandle,
    pub swapchain: &'ctx crate::vulkan::VkSwapchain,
    pub assets: &'ctx crate::assets::AssetManager,
}
