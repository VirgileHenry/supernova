#[derive(Debug)]
pub enum VkError {
    DisplayHande { error: raw_window_handle::HandleError },
    Load { error: ash::LoadingError },
    MissingPhysicalDevice { device_names: Vec<String> },
    Vk { error: ash::vk::Result },
    WindowHandle { error: raw_window_handle::HandleError },
}

impl std::fmt::Display for VkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DisplayHande { error } => write!(f, "Failed to get the display handle: {error}"),
            Self::Load { error } => write!(f, "Failed to load vulkan: {error}"),
            Self::MissingPhysicalDevice { device_names } => {
                let device_names = device_names.join(", ");
                write!(f, "No suitable physical devices among {device_names}")
            }
            Self::Vk { error } => write!(f, "Vulkan error: {error}"),
            Self::WindowHandle { error } => write!(f, "Failed to get the window handle: {error}"),
        }
    }
}

impl std::error::Error for VkError {}

impl From<ash::LoadingError> for VkError {
    fn from(error: ash::LoadingError) -> Self {
        VkError::Load { error }
    }
}

impl From<ash::vk::Result> for VkError {
    fn from(error: ash::vk::Result) -> Self {
        VkError::Vk { error }
    }
}
