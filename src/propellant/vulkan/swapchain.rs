use ash::vk::Handle;

const MAX_FRAMES_IN_FLIGHTS: usize = 2;

pub struct Swapchain {
    pub swapchain_device: ash::khr::swapchain::Device,
    pub swapchain: ash::vk::SwapchainKHR,
    format: ash::vk::Format,
    extent: ash::vk::Extent2D,
    images: Vec<SwapchainImage>,
    synchronization: SwapchainSynchronization,
}

impl Swapchain {
    pub fn create(
        vulkan_state: &crate::propellant::vulkan::VulkanState,
        device: &ash::Device,
        window: &winit::window::Window,
    ) -> Result<Swapchain, crate::ScError> {
        let indices = vulkan_state.queue_family_indices;
        let support = vulkan_state.current_swapchain_support()?;

        let surface_format = Self::get_swapchain_surface_format(&support.formats);
        let present_mode = Self::get_swapchain_present_mode(&support.present_modes);
        let extent = Self::get_swapchain_extent(window, support.capabilities);

        let mut image_count = support.capabilities.min_image_count + 1;
        if support.capabilities.max_image_count != 0 {
            image_count = image_count.min(support.capabilities.max_image_count);
        }

        // if the graphics and presentation are on the same queue family, use exclusive
        let image_sharing_mode = if match indices.present {
            Some((queue_index, _)) => queue_index == indices.graphics.0,
            None => true,
        } {
            ash::vk::SharingMode::EXCLUSIVE
        } else {
            // TODO: this needs to be handled properly, and will shout if some software uses this
            ash::vk::SharingMode::CONCURRENT
        };

        let mut used_family_indices = std::collections::HashSet::with_capacity(4);

        used_family_indices.insert(indices.graphics.0);
        if let Some((index, _)) = indices.present {
            used_family_indices.insert(index);
        }
        if let Some((index, _)) = indices.transfer {
            used_family_indices.insert(index);
        }
        if let Some((index, _)) = indices.compute {
            used_family_indices.insert(index);
        }

        let used_family_indices = used_family_indices.into_iter().collect::<Vec<_>>();

        let create_info = ash::vk::SwapchainCreateInfoKHR {
            surface: vulkan_state.surface,
            min_image_count: image_count,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: ash::vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode,
            queue_family_index_count: used_family_indices.len() as u32,
            p_queue_family_indices: if used_family_indices.is_empty() {
                std::ptr::null()
            } else {
                used_family_indices.as_ptr()
            },
            pre_transform: support.capabilities.current_transform,
            composite_alpha: ash::vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: 1,
            ..Default::default()
        };

        let swapchain_device = ash::khr::swapchain::Device::new(vulkan_state, device);
        let swapchain = unsafe { swapchain_device.create_swapchain(&create_info, None)? };

        let images = unsafe { swapchain_device.get_swapchain_images(swapchain)? };

        let images = images
            .into_iter()
            .map(|image| SwapchainImage::create(device, image, surface_format.format))
            .collect::<Result<Vec<_>, _>>()?;

        let synchronization = SwapchainSynchronization::create(device, images.len())?;

        log::info!(
            "Successefuly created renderer swapchain, with {} images and {} max frames in flight",
            images.len(),
            MAX_FRAMES_IN_FLIGHTS
        );

        Ok(Swapchain {
            swapchain,
            swapchain_device,
            extent,
            format: surface_format.format,
            images,
            synchronization,
        })
    }

    pub fn present(
        &self,
        image_index: usize,
        present_ready: ash::vk::Semaphore,
        present_queue: ash::vk::Queue,
    ) -> Result<bool, crate::ScError> {
        let swapchains = [self.swapchain];
        let image_indices = [image_index as u32];
        let wait_semaphores = [present_ready];
        let present_info = ash::vk::PresentInfoKHR {
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: if wait_semaphores.is_empty() {
                std::ptr::null()
            } else {
                wait_semaphores.as_ptr()
            },
            swapchain_count: swapchains.len() as u32,
            p_swapchains: if swapchains.is_empty() {
                std::ptr::null()
            } else {
                swapchains.as_ptr()
            },
            p_image_indices: image_indices.as_ptr(),
            ..Default::default()
        };

        unsafe {
            self.swapchain_device
                .queue_present(present_queue, &present_info)
                .map_err(crate::ScError::from)
        }
    }

    pub fn extent(&self) -> ash::vk::Extent2D {
        self.extent
    }

    pub fn format(&self) -> ash::vk::Format {
        self.format
    }

    pub fn image_views(&self) -> impl Iterator<Item = &ash::vk::ImageView> {
        self.images.iter().map(|image| &image.view)
    }

    pub fn frame_count(&self) -> usize {
        self.images.len()
    }

    /// Allows to wait fences for resources to be available,
    /// increment frame count, get image index,
    /// and returns the image index as well as a struct containing swapchain related sync objects.
    pub fn go_to_next_frame(&mut self, device: &ash::Device) -> Result<(usize, InflightFrameSync), crate::ScError> {
        let sync = self.synchronization.next_sync();
        let image_index = unsafe {
            self.swapchain_device
                .acquire_next_image(self.swapchain, u64::MAX, sync.image_available, ash::vk::Fence::null())?
                .0 as usize
        };
        if !self.synchronization.in_flight_images[image_index].is_null() {
            unsafe {
                device.wait_for_fences(
                    &[self.synchronization.in_flight_images[image_index], sync.frame_finished],
                    true,
                    u64::MAX,
                )?;
            }
        } else {
            unsafe {
                device.wait_for_fences(&[sync.frame_finished], true, u64::MAX)?;
            }
        }
        unsafe {
            device.reset_fences(&[sync.frame_finished])?;
        }
        self.synchronization.in_flight_images[image_index] = sync.frame_finished;
        Ok((image_index, sync))
    }

    pub fn destroy_swapchain(&mut self, device: &ash::Device) {
        unsafe {
            self.images.iter_mut().for_each(|iv| iv.destroy(device));
            self.swapchain_device.destroy_swapchain(self.swapchain, None);
            self.synchronization.destroy(device);
        }
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

impl Drop for Swapchain {
    fn drop(&mut self) {
        // This is left empty intentionnaly:
        // the vulkan interface of the renderer is responsible for cleaning up the swapchain.
    }
}

/// images of the swapchain, that will get displayed on the screen
struct SwapchainImage {
    _image: ash::vk::Image,
    view: ash::vk::ImageView,
}

impl SwapchainImage {
    fn create(device: &ash::Device, image: ash::vk::Image, format: ash::vk::Format) -> Result<SwapchainImage, crate::ScError> {
        let components = ash::vk::ComponentMapping {
            r: ash::vk::ComponentSwizzle::IDENTITY,
            g: ash::vk::ComponentSwizzle::IDENTITY,
            b: ash::vk::ComponentSwizzle::IDENTITY,
            a: ash::vk::ComponentSwizzle::IDENTITY,
        };
        let subresource_range = ash::vk::ImageSubresourceRange {
            aspect_mask: ash::vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };
        let image_view_create_info = ash::vk::ImageViewCreateInfo {
            image,
            view_type: ash::vk::ImageViewType::TYPE_2D,
            format,
            components,
            subresource_range,
            ..Default::default()
        };

        let view = unsafe { device.create_image_view(&image_view_create_info, None)? };

        Ok(SwapchainImage { _image: image, view })
    }

    fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_image_view(self.view, None);
        }
    }
}

pub struct SwapchainSynchronization {
    current_frame: usize,
    sync: [InflightFrameSync; MAX_FRAMES_IN_FLIGHTS],
    /// Vec of fences that are currently being rendered.
    /// This maps the image index to the fences, not the frame index like the sync.
    /// It allows to handle image out of order, ot limit frames in flight to image number.
    in_flight_images: Vec<ash::vk::Fence>,
}

impl SwapchainSynchronization {
    pub fn create(device: &ash::Device, swapchain_image_count: usize) -> Result<SwapchainSynchronization, crate::ScError> {
        let mut sync = [InflightFrameSync::null(); MAX_FRAMES_IN_FLIGHTS];

        for i in 0..MAX_FRAMES_IN_FLIGHTS {
            sync[i] = InflightFrameSync::create(device)?;
        }

        let in_flight_images = std::iter::repeat(ash::vk::Fence::null())
            .take(swapchain_image_count)
            .collect();

        Ok(SwapchainSynchronization {
            current_frame: 0,
            sync,
            in_flight_images,
        })
    }

    pub fn next_sync(&mut self) -> InflightFrameSync {
        let result = self.sync[self.current_frame];
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHTS;
        result
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        self.sync.iter_mut().for_each(|sync| sync.destroy(device));
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InflightFrameSync {
    pub image_available: ash::vk::Semaphore,
    pub render_finished: ash::vk::Semaphore,
    pub frame_finished: ash::vk::Fence,
}

impl InflightFrameSync {
    pub fn create(device: &ash::Device) -> Result<InflightFrameSync, crate::ScError> {
        let semaphore_create_info = ash::vk::SemaphoreCreateInfo { ..Default::default() };
        let fence_create_info = ash::vk::FenceCreateInfo {
            flags: ash::vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };
        Ok(InflightFrameSync {
            image_available: unsafe { device.create_semaphore(&semaphore_create_info, None)? },
            render_finished: unsafe { device.create_semaphore(&semaphore_create_info, None)? },
            frame_finished: unsafe { device.create_fence(&fence_create_info, None)? },
        })
    }

    pub fn null() -> InflightFrameSync {
        InflightFrameSync {
            image_available: ash::vk::Semaphore::null(),
            render_finished: ash::vk::Semaphore::null(),
            frame_finished: ash::vk::Fence::null(),
        }
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_semaphore(self.image_available, None);
            device.destroy_semaphore(self.render_finished, None);
            device.destroy_fence(self.frame_finished, None);
        }
    }
}
