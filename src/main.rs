pub mod construct;
pub mod consts;
pub mod csg;
pub mod error;
mod propellant;
mod supernova;
pub mod types;
pub mod vulkan;

pub use error::ScError;

fn main() -> Result<(), crate::ScError> {
    env_logger::init();

    let core_event_loop = winit::event_loop::EventLoop::with_user_event().build()?;
    let event_loop_proxy = core_event_loop.create_proxy();

    type Host = propellant::EngineHost<supernova::SupernovaApp>;
    let mut host: Host = propellant::EngineHost::create(event_loop_proxy);
    core_event_loop.run_app(&mut host)?;

    Ok(())
}
