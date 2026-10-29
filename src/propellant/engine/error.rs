use crate::propellant::assets;
use crate::propellant::vulkan;

#[derive(Debug)]
pub enum EngineCreateError {
    AssetManager(assets::AssetLoadError),
    CreateVulkanContext(vulkan::VkError),
    CreateWindow(winit::error::OsError),
}

impl std::fmt::Display for EngineCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AssetManager(error) => write!(f, "Failed to load asset manager: {error}"),
            Self::CreateVulkanContext(error) => write!(f, "Failed to create vulkan context: {error}"),
            Self::CreateWindow(error) => write!(f, "Failed to create window: {error}"),
        }
    }
}

impl std::error::Error for EngineCreateError {}

impl From<assets::AssetLoadError> for EngineCreateError {
    fn from(error: assets::AssetLoadError) -> Self {
        Self::AssetManager(error)
    }
}

impl From<vulkan::VkError> for EngineCreateError {
    fn from(error: vulkan::VkError) -> Self {
        Self::CreateVulkanContext(error)
    }
}

impl From<winit::error::OsError> for EngineCreateError {
    fn from(error: winit::error::OsError) -> Self {
        Self::CreateWindow(error)
    }
}
