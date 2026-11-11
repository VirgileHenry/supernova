//! Supernova is the actual game running on the propellant engine.

pub struct SupernovaApp {}

impl propellant::application::Application for SupernovaApp {
    const NAME: &'static str = "Supernova";
    const VERSION: propellant::version::Version = propellant::version::version(0, 1, 0);

    type Renderer = crate::renderer::Renderer;

    fn create() -> Self {
        Self {}
    }

    fn load_assets(&mut self, assets: &mut propellant::assets::AssetManager) {
        if let Err(e) = assets.load::<crate::assets::SegmentAsset, _>("segments") {
            log::error!("Failed to load segment assets: {e}");
        }
    }

    fn tick(&mut self, _delta_time: std::time::Duration) {}

    fn shutdown(&mut self) {}

    fn start_scene() -> propellant::scene::SceneLoader {
        crate::scenes::test_scene::test_scene_loader()
    }
}
