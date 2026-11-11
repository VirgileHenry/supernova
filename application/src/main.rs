mod assets;
mod components;
mod renderer;
mod scenes;
mod supernova;

fn main() -> std::io::Result<()> {
    /* Info in debug, warning in release. Can be overriden by the env. */
    let default_level = if cfg!(debug_assertions) { "info" } else { "warn" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_level)).init();

    let core_event_loop = propellant::EventLoop::with_user_event()
        .build()
        .map_err(std::io::Error::other)?;
    core_event_loop.set_control_flow(propellant::ControlFlow::Poll);
    let event_loop_proxy = core_event_loop.create_proxy();

    let mut host = propellant::host::EngineHost::<supernova::SupernovaApp>::create(event_loop_proxy);
    core_event_loop.run_app(&mut host).map_err(std::io::Error::other)?;

    Ok(())
}
