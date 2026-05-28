/// Generic trait for applications that the engine can run.
pub trait Application: Sized {
    fn create(
        proxy: &crate::propellant::EventLoopProxy,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Result<Self, crate::ScError>;
    fn window(&self) -> &winit::window::Window;
    fn tick(&mut self);
    fn handle_window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    );
    fn handle_engine_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: crate::propellant::EngineEvent);
}
