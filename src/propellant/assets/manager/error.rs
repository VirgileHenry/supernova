#[derive(Debug)]
pub enum AssetLoadError {
    Io(std::io::Error),
    Vulkan(crate::propellant::vulkan::VkError),
}

impl std::fmt::Display for AssetLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "Io error: {error}"),
            Self::Vulkan(error) => write!(f, "GPU allocation (vulkan) error: {error}"),
        }
    }
}

impl std::error::Error for AssetLoadError {}

impl From<std::io::Error> for AssetLoadError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<crate::propellant::vulkan::VkError> for AssetLoadError {
    fn from(error: crate::propellant::vulkan::VkError) -> Self {
        Self::Vulkan(error)
    }
}
