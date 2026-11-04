use crate::renderer::render_graph::RenderGraphImpl;
use crate::renderer::render_pass::RenderPassCreateContext;
use crate::renderer::render_pass::RenderPassRenderContext;
use ash::prelude::VkResult;

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

    fn create(context: &RenderPassCreateContext) -> Result<Self, Self::CreateError> {
        Ok(Self {
            first: First::create(context).map_err(|e| NodeChainCreateError::First(e))?,
            second: Second::create(context).map_err(|e| NodeChainCreateError::Second(e))?,
        })
    }

    fn render(&mut self, context: &RenderPassRenderContext, input: Self::Input) -> VkResult<Self::Output> {
        let mid = self.first.render(context, input)?;
        let output = self.second.render(context, mid)?;
        Ok(output)
    }
}

pub enum NodeChainCreateError<FirstErr, SecondErr> {
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
