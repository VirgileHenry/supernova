//! Supernova is the actual game running on the propellant engine.

pub struct SupernovaApp {
    window: winit::window::Window,
    vulkan_state: crate::propellant::VulkanState, /* Fixme: in engine, not here ? */

    assets: crate::propellant::AssetManager,

    /* Fixme: better scene management */
    scene: crate::propellant::Scene,
    last_update: std::time::Instant,
}

impl crate::propellant::Application for SupernovaApp {
    fn create(
        proxy: &crate::propellant::EventLoopProxy,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Result<Self, crate::ScError> {
        // TODO: load player preferences as very first task
        // those preferences shall contains stuff as prefered window size (so we can create window directly as full screen) or prefered GPU

        // very first thing is creating the window!
        // we can use the window to quickly loadup a loading / presentation screen using a very simple API like softbuffer
        let window_attributes = winit::window::WindowAttributes::default().with_title(crate::constants::SUPERNOVA_NAME);
        let window = event_loop.create_window(window_attributes)?;

        // let mut loading_screen = loading_screen::LoadingScreen::new(&window)?;
        // this will directly display an image and present it to the window, opening it up.
        // loading_screen.display_image();

        let vulkan_state = crate::propellant::VulkanState::create(&window)?;

        // loading screen is no longer required, as main resources are created and we can proceed with main loop and let the engine manage
        // drop(loading_screen);

        // load start scene
        // Fixme: that's not the main menu, need loading scene to load the game up
        let scene = crate::propellant::Scene::main_menu(&vulkan_state, &window, proxy.clone())?;

        /* Load the assets up */
        let assets = crate::propellant::AssetManager::load("assets")?;

        Ok(Self {
            window,
            vulkan_state,
            scene,
            last_update: std::time::Instant::now(),
            assets,
        })
    }

    fn tick(&mut self) {
        let now = std::time::Instant::now();
        let delta_time = now - self.last_update;
        self.last_update = now;

        self.scene.update(delta_time);
    }

    fn handle_window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::RedrawRequested => self.tick(),
            other => log::debug!("Unhandled window event: {other:?} for window {window_id:?}"),
        }
    }

    fn handle_engine_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: crate::propellant::EngineEvent) {
        match event {
            crate::propellant::EngineEvent::SwapchainRecreationRequest => {
                let event = crate::propellant::SystemEvent::SwapchainRecreationRequest {
                    vulkan_state: &self.vulkan_state,
                    window: &self.window,
                };
                self.scene.send_system_event(event)
            }
        }
    }
}
