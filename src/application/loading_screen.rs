

pub struct LoadingScreen<'a> {
    window: &'a winit::window::Window,
    surface: softbuffer::Surface<&'a winit::window::Window, &'a winit::window::Window>,
}

impl<'a> LoadingScreen<'a> {
    pub fn new(window: &'a winit::window::Window) -> Result<LoadingScreen, crate::ScError> {

        let context = softbuffer::Context::new(window)?;
        let surface = softbuffer::Surface::new(&context, window)?;

        Ok(LoadingScreen {
            window,
            surface,
        })
    }

    /// Displays an image on the referenced window  by accessing the raw underlying buffer and copying the image bytes to it.
    /// TODO: actually load the image and put it intead of a plain color looks like a nice next step. 
    pub fn display_image(&mut self) {
        let size = self.window.inner_size();
        
        match (std::num::NonZeroU32::new(size.width), std::num::NonZeroU32::new(size.height)) {
            (Some(width), Some(height)) => {
                log::info!("Loading image on loading screen with target resolution {}x{}", width, height);
                if let Err(e) = self.surface.resize(width, height) {
                    log::warn!("Failed to present loading image: unable to resize window surface ({e})");
                    return;
                }
                let mut buffer = self.surface.buffer_mut().unwrap();
                for i in 0..size.width * size.height {
                    let red = 255;
                    let green = 100;
                    let blue = 100;
                    let color = blue | (green << 8) | (red << 16);
                    buffer[i as usize] = color;
                }
                match buffer.present() {
                    Ok(_) => {},
                    Err(e) => log::warn!("Failed to present loading image: {e}"),
                }
            },
            _ => log::warn!("Failed to present loading image: invalid surface dimensions: {}x{}", size.width, size.height),
        }
    }
}

