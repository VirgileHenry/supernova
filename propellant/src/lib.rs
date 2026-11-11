//! Propellant is the engine running this game.
//! It's currently using the hecs ECS.

pub mod application;
pub mod assets;
pub mod components;
pub mod engine;
pub mod event;
pub mod host;
pub mod renderer;
pub mod scene;
pub mod utils;
pub mod version;
pub mod vulkan;

pub use vk_shader_macros::include_glsl;

/* Fixme: better export of these */
pub use winit::event_loop::ControlFlow;
pub use winit::event_loop::EventLoop;

const ENGINE_NAME: &'static std::ffi::CStr = c"Propellant";
