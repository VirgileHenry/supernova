//! Propellant is the engine running this game.
//! It's currently using the hecs ECS.

mod application;
mod assets;
mod event;
mod host;
mod renderer;
mod scene;
mod vulkan;

pub use application::Application;
pub use assets::AssetHandle;
pub use assets::AssetManager;
pub use event::EngineEvent;
pub use host::EngineHost;
pub use host::EventLoopProxy;
pub use scene::Scene;
pub use scene::System;
pub use scene::SystemEvent;
pub use scene::UpdateFrequency;
pub use vulkan::VulkanState;
