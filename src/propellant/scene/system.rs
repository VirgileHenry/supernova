pub trait System {
    fn name(&self) -> &'static str;
    fn frequency(&self) -> UpdateFrequency;
    fn update(&mut self, world: &mut hecs::World, delta_time: std::time::Duration);
    fn handle_event(&mut self, event: SystemEvent);
}

pub enum UpdateFrequency {
    /// The system requires to be updated once per frame.
    PerFrame,
    /// The system requires to be updated on a fixed time interval.
    Fixed(std::time::Duration),
}

pub struct SystemMap {
    /// Hash map mapping an id to a system.
    /// There is a very strong requirement that we should ALLWAYS have the value corresponding to the key here!
    inner: std::collections::HashMap<std::any::TypeId, SystemWrapper>,
}

impl SystemMap {
    pub fn new() -> SystemMap {
        SystemMap {
            inner: std::collections::HashMap::new(),
        }
    }

    pub fn add<S: System + 'static>(&mut self, system: S) {
        let prev = self.inner.insert(
            std::any::TypeId::of::<S>(),
            SystemWrapper {
                inner: Box::new(system),
                idle_time: std::time::Duration::ZERO,
            },
        );
        if let Some(_) = prev {
            log::warn!("Inserting system {} overrode an existing element", std::any::type_name::<S>());
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &SystemWrapper> {
        self.inner.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SystemWrapper> {
        self.inner.values_mut()
    }
}

pub struct SystemWrapper {
    inner: Box<dyn System>,
    idle_time: std::time::Duration,
}

impl SystemWrapper {
    pub fn update(&mut self, world: &mut hecs::World, delta_time: std::time::Duration) {
        match self.inner.frequency() {
            UpdateFrequency::PerFrame => self.inner.update(world, delta_time),
            UpdateFrequency::Fixed(timestep) => {
                self.idle_time += delta_time;
                while self.idle_time >= timestep {
                    self.inner.update(world, timestep);
                    self.idle_time -= timestep;
                }
            }
        }
    }

    pub fn handle_event(&mut self, event: SystemEvent) {
        self.inner.handle_event(event);
    }
}

/// Events that are sent to systems.
#[derive(Clone)]
pub enum SystemEvent<'a> {
    SwapchainRecreationRequest {
        vulkan_state: &'a crate::propellant::VkInstance,
        window: &'a winit::window::Window,
    },
}
