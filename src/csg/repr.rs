/// One node as it lives in the GPU buffer. Fixed size, POD, std430-friendly.
///
/// `params` is reinterpreted per opcode — see the `encode` impls for the
/// per-opcode meaning of each slot.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CsgNodeRepr {
    opcode: u32,
    child_count: u32,
    _pad: [u32; 2],
    data: [f32; 12],
}

impl CsgNodeRepr {
    pub fn new(opcode: u32, child_count: u32, data: [f32; 12]) -> Self {
        Self {
            opcode,
            child_count,
            _pad: [0; _],
            data,
        }
    }
}
