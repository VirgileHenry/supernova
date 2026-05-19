//! We use the vk_shader_macro crate to compile glsl code to spir-v bytecode at compile time.
//! The byte code is stored as constants, and then loaded at runtime.
//! Maybe we could write them into files, but embedding them in the binary prevent hijacking the shaders.

pub struct ShaderSource {
    pub entry_point: *const i8,
    pub code: &'static [u32],
}

// TODO: actually write all the shaders and store them in const structs, like the examples

pub const EXAMPLE_VERT: ShaderSource = ShaderSource {
    entry_point: b"main\0".as_ptr() as *const i8,
    code: vk_shader_macros::include_glsl!("src/propellant/renderer/shaders/example.vert.glsl", kind: vert),
};

pub const EXAMPLE_FRAG: ShaderSource = ShaderSource {
    entry_point: b"main\0".as_ptr() as *const i8,
    code: vk_shader_macros::include_glsl!("src/propellant/renderer/shaders/example.frag.glsl", kind: frag),
};
