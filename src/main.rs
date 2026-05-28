mod constants;
mod csg;
mod error;
mod propellant;
mod supernova;

use error::ScError;
type ScResult<T> = Result<T, ScError>;

fn main() -> ScResult<()> {
    /* Info in debug, warning in release. Can be overriden by the env. */
    let default_level = if cfg!(debug_assertions) { "info" } else { "warn" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_level)).init();

    let core_event_loop = winit::event_loop::EventLoop::with_user_event().build()?;
    core_event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let event_loop_proxy = core_event_loop.create_proxy();

    type Host = propellant::EngineHost<supernova::SupernovaApp>;
    let mut host: Host = propellant::EngineHost::create(event_loop_proxy);
    core_event_loop.run_app(&mut host)?;

    Ok(())
}
