//! We use the vk_shader_macro crate to compile glsl code to spir-v bytecode at compile time.
//! The byte code is stored as constants, and then loaded at runtime.
//! Maybe we could write them into files, but embedding them in the binary prevent hijacking the shaders.

pub struct ShaderSource {
    pub entry_point: &'static std::ffi::CStr,
    pub code: &'static [u32],
}

pub const SEGMENT_VERT: ShaderSource = ShaderSource {
    entry_point: c"main",
    #[cfg(debug_assertions)]
    code: vk_shader_macros::include_glsl!("src/propellant/renderer/shaders/segment.vert.glsl", kind: vert, debug, optimize: zero),
    #[cfg(not(debug_assertions))]
    code: vk_shader_macros::include_glsl!("src/propellant/renderer/shaders/segment.vert.glsl", kind: vert),
};

pub const SEGMENT_FRAG: ShaderSource = ShaderSource {
    entry_point: c"main",
    #[cfg(debug_assertions)]
    code: vk_shader_macros::include_glsl!("src/propellant/renderer/shaders/segment.frag.glsl", kind: frag, debug, optimize: zero),
    #[cfg(not(debug_assertions))]
    code: vk_shader_macros::include_glsl!("src/propellant/renderer/shaders/segment.frag.glsl", kind: frag),
};
