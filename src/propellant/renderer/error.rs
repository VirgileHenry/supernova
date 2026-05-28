#[derive(Debug, Clone)]
pub enum RenderError {
    MissingComponent { component: &'static str },
    Vulkan { error: ash::vk::Result },
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingComponent { component } => write!(f, "Missing component \"{component}\""),
            Self::Vulkan { error } => write!(f, "Vulkan error: {error}"),
        }
    }
}

impl From<ash::vk::Result> for RenderError {
    fn from(error: ash::vk::Result) -> Self {
        Self::Vulkan { error }
    }
}
