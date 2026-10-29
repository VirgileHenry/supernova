//! Supernova is the actual game running on the propellant engine.

pub struct SupernovaApp {}

impl crate::propellant::Application for SupernovaApp {
    fn create() -> Self {
        Self {}
    }

    fn tick(&mut self, _delta_time: std::time::Duration) {}

    fn shutdown(&mut self) {}
}
