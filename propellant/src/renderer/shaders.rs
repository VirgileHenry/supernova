//! We use the vk_shader_macro crate to compile glsl code to spir-v bytecode at compile time.
//! The byte code is stored as constants, and then loaded at runtime.
//! Maybe we could write them into files, but embedding them in the binary prevent hijacking the shaders.

pub struct ShaderSource {
    pub entry_point: &'static std::ffi::CStr,
    pub code: &'static [u32],
}

#[macro_export]
macro_rules! frag_shader {
    ($source:literal) => {
        ShaderSource {
            entry_point: c"main",
            #[cfg(debug_assertions)]
            code: ::propellant::include_glsl!($source, kind: frag, debug, optimize: zero),
            #[cfg(not(debug_assertions))]
            code: ::propellant::include_glsl!($source, kind: frag),
        }
    };
}

#[macro_export]
macro_rules! vert_shader {
    ($source:literal) => {
        ShaderSource {
            entry_point: c"main",
            #[cfg(debug_assertions)]
            code: ::propellant::include_glsl!($source, kind: vert, debug, optimize: zero),
            #[cfg(not(debug_assertions))]
            code: ::propellant::include_glsl!($source, kind: vert),
        }
    };
}
