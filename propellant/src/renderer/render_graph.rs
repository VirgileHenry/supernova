mod chain;
mod create_context;
mod node;
mod rg_impl;

pub use create_context::RenderPassCreateContext;
pub use node::RenderGraphNode;
use rg_impl::RenderGraphImpl;

pub trait RenderGraph {
    type CreateError: std::error::Error;
    fn create(context: RenderPassCreateContext) -> Result<Self, Self::CreateError>;
    fn render(
        &mut self,
        vk_device: &crate::vulkan::VkDeviceHandle,
        assets: &crate::assets::AssetManager,
        world: &hecs::World,
        input: RenderGraphInput,
    ) -> ash::prelude::VkResult<RenderGraphOutput>;
}

pub struct RenderGraphInput {
    pub frame_index: usize,
    pub command_buffer: crate::vulkan::InRecordingCommandBuffer,
    pub render_area: ash::vk::Rect2D,
}

pub struct RenderGraphOutput {
    pub command_buffer: crate::vulkan::InRecordingCommandBuffer,
}

impl<T> RenderGraph for T
where
    T: RenderGraphImpl<Input = RenderGraphInput, Output = RenderGraphOutput>,
{
    type CreateError = T::CreateError;
    fn create(context: RenderPassCreateContext) -> Result<Self, Self::CreateError> {
        <T as RenderGraphImpl>::create(context)
    }
    fn render(
        &mut self,
        vk_device: &crate::vulkan::VkDeviceHandle,
        assets: &crate::assets::AssetManager,
        world: &hecs::World,
        input: RenderGraphInput,
    ) -> ash::prelude::VkResult<RenderGraphOutput> {
        T::render(self, vk_device, assets, world, input)
    }
}
