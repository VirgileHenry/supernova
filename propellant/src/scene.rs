mod error;
mod system;

pub use system::System;
pub use system::SystemEvent;
pub use system::UpdateFrequency;

use crate::vulkan;

/// A Scene is the main container for an ECS and multiple systems.
pub struct Scene {
    /// Name for the scene, mostly used for debug purposes.
    name: String,
    /// Mapping of entities to components.
    world: hecs::World,
    /// The system map regroups all the systems running logic on the world.
    systems: system::SystemMap,
}

impl Scene {
    /// Create a new, fully empty Scene.
    /// This is usefull if we want a placeholder, inactive value.
    fn empty(name: String) -> Self {
        log::info!("Created new scene: \"{name}\"");
        Self {
            name,
            world: hecs::World::new(),
            systems: system::SystemMap::new(),
        }
    }

    pub fn load(loader: SceneLoader) -> Self {
        let mut scene = Self::empty(loader.name);
        (loader.loader)(&mut scene);
        scene
    }

    pub fn send_system_event(&mut self, event: SystemEvent) {
        for system in self.systems.iter_mut() {
            system.handle_event(&mut self.world, event.clone())
        }
    }

    pub fn load_system<S: system::System + 'static>(&mut self, system: S) {
        log::info!("Loading system \"{}\" into scene \"{}\".", system.name(), self.name);
        self.systems.add(system);
    }

    pub fn update(&mut self, assets: &crate::assets::AssetManager, delta_time: std::time::Duration) {
        for system in self.systems.iter_mut() {
            system.update(assets, &mut self.world, delta_time);
        }
    }
}

pub struct SceneLoader {
    name: String,
    loader: Box<dyn FnOnce(&mut Scene)>,
}

impl SceneLoader {
    pub fn new<Loader: FnOnce(&mut Scene) + 'static>(name: &str, loader: Loader) -> Self {
        SceneLoader {
            name: name.to_string(),
            loader: Box::new(loader),
        }
    }
}
