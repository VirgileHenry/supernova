#[derive(Debug)]
pub enum AssetLoadError {
    Io(std::io::Error),
    Vulkan(ash::vk::Result),
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

impl From<ash::vk::Result> for AssetLoadError {
    fn from(error: ash::vk::Result) -> Self {
        Self::Vulkan(error)
    }
}
