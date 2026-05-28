use crate::propellant::components::Transform;

/// Uniform for a single segment instance to render.
#[repr(C)]
#[derive(Copy, Clone)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct SegmentInstanceUniform {
    pub transform: [[f32; 4]; 4],
    pub transform_inv: [[f32; 4]; 4],
    pub shell_offset: u32,
    pub shell_length: u32,
    pub interior_offset: u32,
    pub interior_length: u32,
}

impl SegmentInstanceUniform {
    pub fn from_segment_components(transform: &Transform, segment: &crate::propellant::assets::Segment) -> Self {
        SegmentInstanceUniform {
            transform: transform.to_matrix().to_cols_array_2d(),
            transform_inv: transform.to_matrix().inverse().to_cols_array_2d(),
            shell_offset: segment.shell_view().offset() as u32,
            shell_length: segment.shell_view().length() as u32,
            interior_offset: segment.interior_view().offset() as u32,
            interior_length: segment.interior_view().length() as u32,
        }
    }
}
