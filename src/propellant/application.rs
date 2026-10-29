/// Generic trait for applications that the engine can run.
pub trait Application: Sized {
    /// Create a new instance of the application from the event loop and its proxy.
    fn create() -> Self;

    /// Application tick, called before engine ticks and runs all the systems.
    /// Entry point of user code.
    fn tick(&mut self, delta_time: std::time::Duration);

    /// Called when the engine is going to shutdown
    fn shutdown(&mut self);
}
