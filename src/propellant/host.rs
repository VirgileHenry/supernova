pub type EventLoopProxy = winit::event_loop::EventLoopProxy<crate::propellant::event::EngineEvent>;

pub struct EngineHost<App: crate::propellant::Application> {
    event_loop_proxy: EventLoopProxy,
    engine: Option<crate::propellant::Engine<App>>,
}

impl<App: crate::propellant::Application> EngineHost<App> {
    pub fn create(event_loop_proxy: EventLoopProxy) -> Self {
        EngineHost {
            event_loop_proxy,
            engine: None,
        }
    }

    fn init(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Result<(), crate::propellant::engine::EngineCreateError> {
        let engine = crate::propellant::Engine::create(&self.event_loop_proxy, event_loop)?;
        self.engine = Some(engine);
        Ok(())
    }
}

impl<App: crate::propellant::Application> winit::application::ApplicationHandler<crate::propellant::EngineEvent>
    for EngineHost<App>
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        match self.engine.as_ref() {
            None => {
                log::info!("Application resumed with no app data, creating the application");
                /* Initialize the application */
                if let Err(e) = self.init(event_loop) {
                    log::error!("Failed to init resources: {e}");
                    event_loop.exit();
                }
            }
            Some(_) => log::warn!("Received resume event with all resources already created."),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match self.engine.as_mut() {
            Some(engine) => engine.handle_window_event(event_loop, window_id, event),
            None => log::warn!("Unable to handle window event: application not loaded!"),
        }
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: crate::propellant::EngineEvent) {
        match self.engine.as_mut() {
            Some(application) => application.handle_engine_event(event_loop, event),
            None => log::warn!("Unable to handle window event: application not loaded!"),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        match self.engine.as_mut() {
            Some(engine) => engine.request_redraw(),
            None => {}
        }
    }
}
