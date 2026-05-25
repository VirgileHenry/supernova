mod image;

/// Interface to use the Vulkan swapchain.
pub struct VkSwapchain {
    /// Reference to the Vulkan device.
    vk_device: crate::propellant::vulkan::VkDeviceHandle,

    /// Vulkan swapchain device
    swapchain_device: ash::khr::swapchain::Device,
    /// Vulkan swapchain
    swapchain: ash::vk::SwapchainKHR,

    /// Current format of the swapchain
    format: ash::vk::Format,
    /// Current extent of the swapchain
    extent: ash::vk::Extent2D,

    /// Images of the swapchains.
    images: Vec<image::VkSwapchainImage>,
}

impl VkSwapchain {
    pub fn create(
        vk_instance: &crate::propellant::VkInstance,
        vk_device: &crate::propellant::vulkan::VkDeviceHandle,
        window: &winit::window::Window,
        previous: Option<&Self>,
    ) -> Result<VkSwapchain, crate::ScError> {
        let indices = vk_device.physical_device().queue_family_indices();
        let surface_support = vk_device.physical_device().surface_support();
        let surface_capabilities = vk_instance.surface_capabilities(vk_device.physical_device())?;

        let surface_format = Self::get_swapchain_surface_format(&surface_support.formats());
        let present_mode = Self::get_swapchain_present_mode(&surface_support.present_modes());
        let extent = Self::get_swapchain_extent(window, surface_capabilities);

        let mut image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count != 0 {
            image_count = image_count.min(surface_capabilities.max_image_count);
        }

        let mut used_family_indices = std::collections::HashSet::new();

        used_family_indices.insert(indices.graphics);
        used_family_indices.insert(indices.transfer);
        used_family_indices.insert(indices.compute);

        let used_family_indices = used_family_indices.into_iter().collect::<Vec<_>>();

        let create_info = ash::vk::SwapchainCreateInfoKHR::default()
            .surface(vk_instance.surface())
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(ash::vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(used_family_indices.as_slice())
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(ash::vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let create_info = match previous {
            None => create_info,
            Some(previous_swapchain) => create_info.old_swapchain(previous_swapchain.swapchain),
        };

        let swapchain_device = ash::khr::swapchain::Device::new(vk_instance, vk_device);
        let swapchain = unsafe { swapchain_device.create_swapchain(&create_info, None)? };

        let images = unsafe { swapchain_device.get_swapchain_images(swapchain)? };

        let images = images
            .into_iter()
            .map(|image| image::VkSwapchainImage::create(vk_device, image, surface_format.format))
            .collect::<Result<Vec<_>, _>>()?;

        log::info!(
            "Successefuly created renderer swapchain, with {} images and {} max frames in flight",
            images.len(),
            crate::propellant::vulkan::MAX_FRAMES_IN_FLIGHTS
        );

        Ok(VkSwapchain {
            vk_device: vk_device.clone(),
            swapchain,
            swapchain_device,
            extent,
            format: surface_format.format,
            images,
        })
    }

    pub fn present(
        &self,
        image_index: usize,
        present_ready: ash::vk::Semaphore,
        present_queue: ash::vk::Queue,
    ) -> ash::prelude::VkResult<bool> {
        let swapchains = [self.swapchain];
        let image_indices = [image_index as u32];
        let wait_semaphores = [present_ready];

        let present_info = ash::vk::PresentInfoKHR::default()
            .wait_semaphores(wait_semaphores.as_slice())
            .swapchains(swapchains.as_slice())
            .image_indices(image_indices.as_slice());

        unsafe { self.swapchain_device.queue_present(present_queue, &present_info) }
    }

    pub fn extent(&self) -> ash::vk::Extent2D {
        self.extent
    }

    pub fn format(&self) -> ash::vk::Format {
        self.format
    }

    pub fn image_views(&self) -> Vec<ash::vk::ImageView> {
        self.images.iter().map(|image| image.view()).collect()
    }

    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// Get the index of the next image.
    /// The image index returned is the index of the image we will use.
    ///
    /// However, the image might not be vailable yet.
    /// The GPU will signal the image_available semaphore when it will be ready.
    pub fn acquire_next_image(
        &self,
        image_available: ash::vk::Semaphore,
    ) -> ash::prelude::VkResult<VkSwapchainImageAcquireResult> {
        let (image_index, suboptimal) = unsafe {
            let fence = ash::vk::Fence::null();
            let device = &self.swapchain_device;
            device.acquire_next_image(self.swapchain, u64::MAX, image_available, fence)?
        };
        Ok(VkSwapchainImageAcquireResult {
            image_index: image_index as usize,
            suboptimal,
        })
    }

    fn get_swapchain_surface_format(formats: &[ash::vk::SurfaceFormatKHR]) -> ash::vk::SurfaceFormatKHR {
        // TODO: better way to get preferred format :)
        formats
            .iter()
            .cloned()
            .find(|f| f.format == ash::vk::Format::B8G8R8A8_SRGB && f.color_space == ash::vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .unwrap_or_else(|| formats[0]) // TODO: handle empty slice of formats (can it even happen ?)
    }

    fn get_swapchain_present_mode(present_modes: &[ash::vk::PresentModeKHR]) -> ash::vk::PresentModeKHR {
        // put the prefered present mode here, in order. First elements are the prefered ones.
        const PREFERED_PRESENT_MODES: [ash::vk::PresentModeKHR; 2] =
            [ash::vk::PresentModeKHR::MAILBOX, ash::vk::PresentModeKHR::FIFO_RELAXED];
        const GUARANTED_FALLBACK_MODE: ash::vk::PresentModeKHR = ash::vk::PresentModeKHR::FIFO;

        PREFERED_PRESENT_MODES
            .iter()
            .cloned()
            .filter(|present_mode| present_modes.contains(&present_mode))
            .next()
            .unwrap_or(GUARANTED_FALLBACK_MODE)
    }

    fn get_swapchain_extent(window: &winit::window::Window, capabilities: ash::vk::SurfaceCapabilitiesKHR) -> ash::vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            ash::vk::Extent2D {
                width: window
                    .inner_size()
                    .width
                    .clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width),
                height: window
                    .inner_size()
                    .height
                    .clamp(capabilities.min_image_extent.height, capabilities.max_image_extent.height),
            }
        }
    }
}

impl Drop for VkSwapchain {
    fn drop(&mut self) {
        unsafe {
            self.images.iter_mut().for_each(|iv| iv.destroy(&self.vk_device));
            self.swapchain_device.destroy_swapchain(self.swapchain, None);
        }
    }
}

pub struct VkSwapchainImageAcquireResult {
    pub image_index: usize,
    pub suboptimal: bool,
}
