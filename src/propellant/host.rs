pub type EventLoopProxy = winit::event_loop::EventLoopProxy<crate::propellant::event::EngineEvent>;

pub struct EngineHost<App: crate::propellant::Application> {
    event_loop_proxy: EventLoopProxy,
    application_data: Option<App>,
}

impl<App: crate::propellant::Application> EngineHost<App> {
    pub fn create(event_loop_proxy: EventLoopProxy) -> Self {
        EngineHost {
            event_loop_proxy,
            application_data: None,
        }
    }

    fn init(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) -> Result<(), crate::ScError> {
        let application_data = App::create(&self.event_loop_proxy, event_loop)?;
        self.application_data = Some(application_data);
        Ok(())
    }
}

impl<App: crate::propellant::Application> winit::application::ApplicationHandler<crate::propellant::EngineEvent>
    for EngineHost<App>
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        /* Initialize the application */
        match self.application_data.as_ref() {
            None => {
                if let Err(e) = self.init(event_loop) {
                    log::error!("Failed to init resources: {e}");
                    event_loop.exit();
                }
            }
            Some(_) => log::warn!("Received resume event with all resources already created."),
        }
        /* Set event loop to poll */
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match self.application_data.as_mut() {
            Some(application) => application.handle_window_event(event_loop, window_id, event),
            None => log::warn!("Unable to handle window event: application not loaded!"),
        }
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: crate::propellant::EngineEvent) {
        match self.application_data.as_mut() {
            Some(application) => application.handle_engine_event(event_loop, event),
            None => log::warn!("Unable to handle window event: application not loaded!"),
        }
    }
}
