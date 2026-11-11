mod binding;
mod node;
mod node_chain;

pub use binding::Binding;
pub use binding::BindingStage;
pub use binding::Bindings;
pub use node::RenderGraphNode;
pub use node_chain::NodeChain;

use crate::renderer::render_pass::RenderPassCreateContext;
use crate::renderer::render_pass::RenderPassRenderContext;
use ash::prelude::VkResult;

#[macro_export]
macro_rules! renderer {
    () => {
            compile_error!("renderer! needs at least one render pass")
        };
        ($pass:ty $(,)?) => {
            $crate::renderer::RenderGraphNode<$pass>
        };
        ($pass:ty, $($rest:ty),+ $(,)?) => {
            $crate::renderer::NodeChain<
                $crate::renderer::RenderGraphNode<$pass>,
                $crate::renderer!($($rest),+)
            >
        };
}

pub trait RenderGraph: Sized {
    fn create(context: &RenderPassCreateContext) -> Result<Self, RenderGraphCreateError>;
    fn render(&mut self, context: &RenderPassRenderContext, input: RenderGraphInput) -> VkResult<RenderGraphOutput>;
}

pub struct RenderGraphInput;
pub struct RenderGraphOutput;

pub trait RenderGraphImpl: Sized {
    type Input;
    type Output;
    type CreateError: std::error::Error;

    fn create(context: &RenderPassCreateContext) -> Result<Self, Self::CreateError>;
    fn render(&mut self, context: &RenderPassRenderContext, input: Self::Input) -> VkResult<Self::Output>;
}

impl<T> RenderGraph for T
where
    T: RenderGraphImpl<Input = RenderGraphInput, Output = RenderGraphOutput>,
{
    fn create(context: &RenderPassCreateContext) -> Result<Self, RenderGraphCreateError> {
        <T as RenderGraphImpl>::create(context).map_err(|e| RenderGraphCreateError(e.to_string()))
    }
    fn render(&mut self, context: &RenderPassRenderContext, input: RenderGraphInput) -> VkResult<RenderGraphOutput> {
        T::render(self, context, input)
    }
}

/// Error when creating the render graph.
/// This is a hard allocated error message.
#[derive(Debug)]
pub struct RenderGraphCreateError(String);

impl std::fmt::Display for RenderGraphCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to create render graph: {}", self.0)
    }
}

impl std::error::Error for RenderGraphCreateError {}
