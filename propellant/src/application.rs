/// Generic trait for applications that the engine can run.
pub trait Application: Sized {
    const NAME: &'static str;
    const VERSION: crate::version::Version;

    type Renderer: crate::renderer::RenderGraph;

    /// Create a new instance of the application from the event loop and its proxy.
    fn create() -> Self;

    /// Function called by the engine to allow the application to load assets.
    fn load_assets(&mut self, assets: &mut crate::assets::AssetManager);

    /// Application tick, called before engine ticks and runs all the systems.
    /// Entry point of user code.
    fn tick(&mut self, delta_time: std::time::Duration);

    /// Called when the engine is going to shutdown
    fn shutdown(&mut self);

    /// The first scene to load in the engine
    fn start_scene() -> crate::scene::SceneLoader;
}
