use crate::renderer::render_graph::RenderGraphCreateError;

#[derive(Debug)]
pub enum RendererCreateError {
    RenderGraph { error: RenderGraphCreateError },
    Vulkan { error: ash::vk::Result },
}

impl std::fmt::Display for RendererCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RenderGraph { error } => write!(f, "Render graph error: {error}"),
            Self::Vulkan { error } => write!(f, "Vulkan error: {error}"),
        }
    }
}

impl From<RenderGraphCreateError> for RendererCreateError {
    fn from(error: RenderGraphCreateError) -> Self {
        Self::RenderGraph { error }
    }
}

impl From<ash::vk::Result> for RendererCreateError {
    fn from(error: ash::vk::Result) -> Self {
        Self::Vulkan { error }
    }
}
