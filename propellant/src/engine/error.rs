use crate::assets;
use crate::vulkan;

#[derive(Debug)]
pub enum EngineCreateError {
    AssetManagerError(assets::AssetLoadError),
    ContextCreateError(vulkan::VulkanContextCreateError),
    CreateWindowError(winit::error::OsError),
}

impl std::fmt::Display for EngineCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AssetManagerError(error) => write!(f, "Failed to load asset manager: {error}"),
            Self::ContextCreateError(error) => write!(f, "Failed to create vulkan context: {error}"),
            Self::CreateWindowError(error) => write!(f, "Failed to create window: {error}"),
        }
    }
}

impl std::error::Error for EngineCreateError {}

impl From<assets::AssetLoadError> for EngineCreateError {
    fn from(error: assets::AssetLoadError) -> Self {
        Self::AssetManagerError(error)
    }
}

impl From<vulkan::VulkanContextCreateError> for EngineCreateError {
    fn from(error: vulkan::VulkanContextCreateError) -> Self {
        Self::ContextCreateError(error)
    }
}

impl From<winit::error::OsError> for EngineCreateError {
    fn from(error: winit::error::OsError) -> Self {
        Self::CreateWindowError(error)
    }
}
