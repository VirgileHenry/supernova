#[derive(Debug)]
pub enum ScError {
    VulkanLoading(ash::LoadingError),
    Vulkan(ash::vk::Result),
    Generic(&'static str),
    OsError(winit::error::OsError),
    EventLoop(winit::error::EventLoopError),
    SoftBuffer(softbuffer::SoftBufferError),
    Io(std::io::Error),
}

impl std::fmt::Display for ScError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScError::VulkanLoading(error) => write!(f, "VulkanLoading error: {error}"),
            ScError::Vulkan(error) => write!(f, "Vulkan error: {error}"),
            ScError::Generic(error) => write!(f, "Generic error: {error}"),
            ScError::OsError(error) => write!(f, "OsError error: {error}"),
            ScError::EventLoop(error) => write!(f, "EventLoop error: {error}"),
            ScError::SoftBuffer(error) => write!(f, "SoftBuffer error: {error}"),
            ScError::Io(error) => write!(f, "I/O error: {error}"),
        }
    }
}

impl From<ash::vk::Result> for ScError {
    #[track_caller]
    fn from(value: ash::vk::Result) -> Self {
        let caller_location = std::panic::Location::caller();
        log::error!("Building vulkan error from failure at {caller_location}");
        ScError::Vulkan(value)
    }
}

impl From<ash::LoadingError> for ScError {
    #[track_caller]
    fn from(value: ash::LoadingError) -> Self {
        let caller_location = std::panic::Location::caller();
        log::error!("Building vulkan loading error from failure at {caller_location}");
        ScError::VulkanLoading(value)
    }
}

impl From<&'static str> for ScError {
    #[track_caller]
    fn from(value: &'static str) -> Self {
        let caller_location = std::panic::Location::caller();
        log::error!("Building generic error from failure at {caller_location}");
        ScError::Generic(value)
    }
}

impl From<winit::error::OsError> for ScError {
    #[track_caller]
    fn from(value: winit::error::OsError) -> Self {
        let caller_location = std::panic::Location::caller();
        log::error!("Building winit OS error from failure at {caller_location}");
        ScError::OsError(value)
    }
}

impl From<softbuffer::SoftBufferError> for ScError {
    #[track_caller]
    fn from(value: softbuffer::SoftBufferError) -> Self {
        let caller_location = std::panic::Location::caller();
        log::error!("Building SoftBuffer error from failure at {caller_location}");
        ScError::SoftBuffer(value)
    }
}

impl From<winit::error::EventLoopError> for ScError {
    #[track_caller]
    fn from(value: winit::error::EventLoopError) -> Self {
        let caller_location = std::panic::Location::caller();
        log::error!("Building event loop error from failure at {caller_location}");
        ScError::EventLoop(value)
    }
}

impl From<std::io::Error> for ScError {
    #[track_caller]
    fn from(value: std::io::Error) -> Self {
        let caller_location = std::panic::Location::caller();
        log::error!("Building io error from failure at {caller_location}");
        ScError::Io(value)
    }
}
