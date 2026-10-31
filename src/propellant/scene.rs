mod error;
mod system;

pub use system::System;
pub use system::SystemEvent;
pub use system::UpdateFrequency;

use crate::propellant::vulkan;

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

    /// Create the Scene used as the main menu.
    pub fn test_scene(
        vk_context: &vulkan::VulkanContext,
        event_loop_proxy: crate::propellant::EventLoopProxy,
        assets: &crate::propellant::AssetManager,
    ) -> Self {
        let mut scene = Self::empty("Test Scene".to_string());

        /* Add a camera holding entity */
        use crate::propellant::components;
        let camera = components::Camera::main(1.0);
        let cam_transform = components::Transform::IDENTITY
            .at(glam::vec3(0., 2., 8.))
            .look_at(glam::Vec3::ZERO);
        scene.world.spawn((camera, cam_transform));

        /* Add a single segment entity */
        match assets.get_segment_handle("tube_small") {
            Some(segment) => {
                let segment_component = components::SegmentRenderer::new(segment);
                let segment_transform = components::Transform::IDENTITY;
                scene.world.spawn((segment_component, segment_transform));
                let segment_component = components::SegmentRenderer::new(segment);
                let segment_transform = components::Transform::IDENTITY.at(glam::vec3(2.0, 1.0, 0.0));
                scene.world.spawn((segment_component, segment_transform));
                let segment_component = components::SegmentRenderer::new(segment);
                let segment_transform = components::Transform::IDENTITY.at(glam::vec3(1.0, 0.0, 2.0));
                scene.world.spawn((segment_component, segment_transform));
            }
            None => log::warn!("Failed to load objects for test scene: missing asset \"tube_small\""),
        }

        /* Standard systems for the menu world */
        match crate::propellant::renderer::Renderer::create(vk_context, event_loop_proxy, assets) {
            Ok(renderer) => scene.load_system(renderer),
            Err(error) => log::error!("Failed to create renderer for test scene: {error}"),
        }

        /* Fixme: little spinner as I implement raymarching */
        scene.load_system(Spinner { speed: 0.5 });

        // Insert all objects for menu world

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

    pub fn update(&mut self, assets: &crate::propellant::assets::AssetManager, delta_time: std::time::Duration) {
        for system in self.systems.iter_mut() {
            system.update(assets, &mut self.world, delta_time);
        }
    }
}

/// Fixme: delete, for the test
struct Spinner {
    speed: f32,
}

impl System for Spinner {
    fn name(&self) -> &'static str {
        "Spinner"
    }

    fn frequency(&self) -> UpdateFrequency {
        UpdateFrequency::PerFrame
    }

    fn handle_event(&mut self, _: &mut hecs::World, _: SystemEvent) {}

    fn update(&mut self, _: &crate::propellant::assets::AssetManager, world: &mut hecs::World, delta_time: std::time::Duration) {
        use crate::propellant::components;

        for (_, (transform, _)) in world.query_mut::<(&mut components::Transform, &components::SegmentRenderer)>() {
            transform.rotate_around(glam::Vec3::Y, self.speed * delta_time.as_secs_f32());
        }
    }
}
