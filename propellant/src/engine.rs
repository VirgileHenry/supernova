mod error;

pub use error::EngineCreateError;

pub struct Engine<App: crate::application::Application> {
    /// User application hosted by the engine
    application: App,

    /// Scene manager
    scene: crate::scene::Scene, /* Fixme: better scene management */
    /// Renderer in use for the engine
    renderer: crate::renderer::Renderer<App::Renderer>,
    /// Asset manager
    assets: crate::assets::AssetManager,
    /// Access to the graphics stack of the engine
    /// WARNING: this shall be dropped after the scene and assets
    vk_context: crate::vulkan::VulkanContext,

    /// Instant of the last update to keep track of delta times
    last_update: std::time::Instant,
}

impl<App: crate::application::Application> Engine<App> {
    pub fn create(
        proxy: &crate::event::EventLoopProxy, /* fixme: in event ? */
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Result<Self, EngineCreateError> {
        // TODO: load player preferences as very first task
        // those preferences shall contains stuff as prefered window size (so we can create window directly as full screen) or prefered GPU

        let mut application = App::create();

        // very first thing is creating the window!
        // we can use the window to quickly loadup a loading / presentation screen using a very simple API like softbuffer
        let window_attributes = winit::window::WindowAttributes::default().with_title(App::NAME);
        let window = event_loop.create_window(window_attributes)?;

        // let mut loading_screen = loading_screen::LoadingScreen::new(&window)?;
        // this will directly display an image and present it to the window, opening it up.
        // loading_screen.display_image();

        let vk_context = crate::vulkan::VulkanContext::create(window, App::NAME, App::VERSION)?;

        /* Load the assets up */
        let mut assets = crate::assets::AssetManager::new(vk_context.device_handle(), "assets")?;
        application.load_assets(&mut assets);

        /* Create the renderer */
        let renderer = crate::renderer::Renderer::create(&vk_context, proxy.clone(), &assets)?;

        // loading screen is no longer required, as main resources are created and we can proceed with main loop and let the engine manage
        // drop(loading_screen);

        // load start scene
        // Fixme: that's not the main menu, need loading scene to load the game up
        let scene = crate::scene::Scene::load(App::start_scene(), &assets);

        Ok(Self {
            application,
            scene,
            assets,
            renderer,
            vk_context,
            last_update: std::time::Instant::now(),
        })
    }

    pub fn handle_window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::RedrawRequested => {
                /* Engine main loop: should it be on redraw requested ? */
                let now = std::time::Instant::now();
                let delta_time = now - self.last_update;
                self.last_update = now;

                self.application.tick(delta_time);
                self.scene.update(&self.assets, delta_time);

                if let Err(e) = self.renderer.render(&self.assets, &self.scene.world()) {
                    log::warn!("Failed to render scene: {e}");
                }
            }
            winit::event::WindowEvent::Resized(new_size) => {
                use crate::components::Camera;
                use crate::components::camera::MainCamera;

                /* Set the aspect ratio of each main camera */
                for (_, main_cam) in self.scene.world().query::<&mut Camera<MainCamera>>().iter() {
                    main_cam.resize(new_size.width, new_size.height);
                }
            }
            other => log::debug!("Unhandled window event: {other:?} for window {window_id:?}"),
        }
    }

    pub fn handle_engine_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: crate::event::EngineEvent) {
        match event {
            crate::event::EngineEvent::SwapchainRecreationRequest => {
                log::warn!("Ignoring swapchain recreation request")
                // let event = crate::SystemEvent::SwapchainRecreationRequest {
                //     vk_context: &self.vk_context,
                //     assets: &self.assets,
                // };
                // self.scene.send_system_event(event)
            }
        }
    }

    pub fn request_redraw(&self) {
        self.vk_context.request_redraw();
    }
}
