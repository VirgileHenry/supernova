use crate::propellant::components;

/// Uniform data to be sent to the GPU for the camera.
#[repr(C)]
#[derive(Copy, Clone)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn from_camera_components<Usage: components::camera::CameraUsage>(
        transform: &components::Transform,
        camera: &components::Camera<Usage>,
    ) -> Self {
        Self {
            view: transform.to_matrix().inverse().to_cols_array_2d(),
            proj: camera.projection_matrix().to_cols_array_2d(),
        }
    }
}
