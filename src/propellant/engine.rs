mod error;

pub use error::EngineCreateError;

pub struct Engine<App: crate::propellant::Application> {
    /// User application hosted by the engine
    application: App,

    /// Scene manager
    scene: crate::propellant::Scene, /* Fixme: better scene management */
    /// Asset manager
    assets: crate::propellant::AssetManager,
    /// Access to the graphics stack of the engine
    /// WARNING: this shall be dropped after the scene and assets
    vk_context: crate::propellant::vulkan::VulkanContext,

    /// Instant of the last update to keep track of delta times
    last_update: std::time::Instant,
}

impl<App: crate::propellant::Application> Engine<App> {
    pub fn create(
        proxy: &crate::propellant::EventLoopProxy,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Result<Self, EngineCreateError> {
        // TODO: load player preferences as very first task
        // those preferences shall contains stuff as prefered window size (so we can create window directly as full screen) or prefered GPU

        // very first thing is creating the window!
        // we can use the window to quickly loadup a loading / presentation screen using a very simple API like softbuffer
        let window_attributes = winit::window::WindowAttributes::default().with_title(crate::constants::SUPERNOVA_NAME);
        let window = event_loop.create_window(window_attributes)?;

        // let mut loading_screen = loading_screen::LoadingScreen::new(&window)?;
        // this will directly display an image and present it to the window, opening it up.
        // loading_screen.display_image();

        let vk_context = crate::propellant::vulkan::VulkanContext::create(window)?;

        /* Load the assets up */
        let assets = crate::propellant::AssetManager::load(vk_context.device_handle(), "assets")?;

        // loading screen is no longer required, as main resources are created and we can proceed with main loop and let the engine manage
        // drop(loading_screen);

        // load start scene
        // Fixme: that's not the main menu, need loading scene to load the game up
        let scene = crate::propellant::Scene::test_scene(&vk_context, proxy.clone(), &assets);

        let application = App::create();

        Ok(Self {
            vk_context,
            scene,
            last_update: std::time::Instant::now(),
            assets,
            application,
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
            }
            winit::event::WindowEvent::Resized(new_size) => {
                use crate::propellant::SystemEvent;
                self.scene.send_system_event(SystemEvent::SwapchainRecreationRequest {
                    vk_context: &self.vk_context,
                });
                self.scene.send_system_event(SystemEvent::WindowResize {
                    width: new_size.width,
                    height: new_size.height,
                });
            }
            other => log::debug!("Unhandled window event: {other:?} for window {window_id:?}"),
        }
    }

    pub fn handle_engine_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        event: crate::propellant::EngineEvent,
    ) {
        match event {
            crate::propellant::EngineEvent::SwapchainRecreationRequest => {
                let event = crate::propellant::SystemEvent::SwapchainRecreationRequest {
                    vk_context: &self.vk_context,
                };
                self.scene.send_system_event(event)
            }
        }
    }

    pub fn request_redraw(&self) {
        self.vk_context.request_redraw();
    }
}
