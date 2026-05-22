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
    pub fn empty() -> Self {
        Self {
            name: "Empty Scene".to_string(),
            world: hecs::World::new(),
            systems: system::SystemMap::new(),
        }
    }

    /// Create the Scene used as the main menu.
    pub fn main_menu(
        vulkan_state: &crate::propellant::vulkan::VulkanState,
        window: &winit::window::Window,
        event_loop_proxy: crate::propellant::EventLoopProxy,
    ) -> Result<Self, crate::ScError> {
        log::info!("Creating menu Scene");

        let world = hecs::World::new();
        let mut systems = system::SystemMap::new();

        /* Standard systems for the menu world */
        systems.add(crate::propellant::renderer::Renderer::create(
            vulkan_state,
            window,
            event_loop_proxy,
        )?);

        // Insert all objects for menu world

        Ok(Scene {
            name: "Main Menu".to_string(),
            world,
            systems,
        })
    }

    pub fn send_system_event(&mut self, event: SystemEvent) {
        for system in self.systems.iter_mut() {
            system.handle_event(event.clone())
        }
    }

    pub fn load_system<S: system::System + 'static>(&mut self, system: S) {
        log::info!("Loaded system {} into Scene.", system.name());
        self.systems.add(system);
    }

    pub fn update(&mut self, delta_time: std::time::Duration) {
        for system in self.systems.iter_mut() {
            system.update(&mut self.world, delta_time);
        }
    }
}
