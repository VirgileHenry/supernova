use crate::renderer::render_graph::RenderGraphImpl;

pub struct NodeChain<First, Second>
where
    First: RenderGraphImpl,
    Second: RenderGraphImpl<Input = First::Output>,
{
    first: First,
    second: Second,
}

impl<First, Second> RenderGraphImpl for NodeChain<First, Second>
where
    First: RenderGraphImpl,
    Second: RenderGraphImpl<Input = First::Output>,
{
    type Input = First::Input;
    type Output = Second::Output;
    type CreateError = NodeChainCreateError<First::CreateError, Second::CreateError>;

    fn create(context: crate::renderer::render_graph::RenderPassCreateContext) -> Result<Self, Self::CreateError> {
        Ok(Self {
            first: First::create(context)?,
            second: Second::create(context)?,
        })
    }

    fn render(
        &mut self,
        vk_device: &crate::vulkan::VkDeviceHandle,
        assets: &crate::assets::AssetManager,
        world: &hecs::World,
        input: Self::Input,
    ) -> ash::prelude::VkResult<Self::Output> {
        let mid = self.first.render(vk_device, assets, world, input)?;
        let output = self.second.render(vk_device, assets, world, mid)?;
        Ok(output)
    }
}

enum NodeChainCreateError<FirstErr, SecondErr> {
    First(FirstErr),
    Second(SecondErr),
}

impl<FirstErr, SecondErr> std::fmt::Debug for NodeChainCreateError<FirstErr, SecondErr>
where
    FirstErr: std::fmt::Debug,
    SecondErr: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::First(err) => err.fmt(f),
            Self::Second(err) => err.fmt(f),
        }
    }
}

impl<FirstErr, SecondErr> std::fmt::Display for NodeChainCreateError<FirstErr, SecondErr>
where
    FirstErr: std::fmt::Display,
    SecondErr: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::First(err) => err.fmt(f),
            Self::Second(err) => err.fmt(f),
        }
    }
}

impl<FirstErr, SecondErr> std::error::Error for NodeChainCreateError<FirstErr, SecondErr>
where
    FirstErr: std::error::Error,
    SecondErr: std::error::Error,
{
}
