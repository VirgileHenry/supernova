mod system;

pub use system::System;
pub use system::SystemEvent;
pub use system::UpdateFrequency;

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
        Self {
            name,
            world: hecs::World::new(),
            systems: system::SystemMap::new(),
        }
    }

    /// Create the Scene used as the main menu.
    pub fn test_scene(
        vk_context: &crate::propellant::VkInstance,
        vk_device: crate::propellant::vulkan::VkDeviceHandle,
        window: &winit::window::Window,
        event_loop_proxy: crate::propellant::EventLoopProxy,
        assets: &crate::propellant::AssetManager,
    ) -> Result<Self, crate::ScError> {
        log::info!("Creating test scene...");

        let mut scene = Self::empty("Test Scene".to_string());

        /* Add a camera holding entity */
        use crate::propellant::components;
        let camera = components::Camera::primary(1.0);
        let cam_transform = components::Transform::IDENTITY.at(glam::vec3(0., 0., 3.));
        scene.world.spawn((camera, cam_transform));

        /* Add a single segment entity */
        match assets.get_segment_handle("tube_small") {
            Some(segment) => {
                let segment_component = components::SegmentRenderer::new(segment);
                let segment_transform = components::Transform::IDENTITY;
                scene.world.spawn((segment_component, segment_transform));
            }
            None => log::warn!("Failed to load objects for test scene: missing asset \"tube_small\""),
        }

        /* Standard systems for the menu world */
        scene.load_system(crate::propellant::renderer::Renderer::create(
            vk_context,
            vk_device,
            window,
            event_loop_proxy,
        )?);

        // Insert all objects for menu world

        Ok(scene)
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

    pub fn update(&mut self, assets: &crate::propellant::assets::AssetManager, delta_time: std::time::Duration) {
        for system in self.systems.iter_mut() {
            system.update(assets, &mut self.world, delta_time);
        }
    }
}
