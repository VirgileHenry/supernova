#[derive(Debug, Clone, Copy)]
pub struct Camera<const MAIN: bool> {
    fov_y_radians: f32,
    aspect_ratio: f32,
    z_near: f32,
    z_far: f32,
}

impl Camera<true> {
    /// Create a new primary (main) camera.
    pub fn primary(aspect_ratio: f32) -> Self {
        Self {
            fov_y_radians: 60.0_f32.to_radians(),
            aspect_ratio,
            z_near: 0.1,
            z_far: 1000.0,
        }
    }
}

impl Camera<false> {
    /// Create a new secondary camera.
    pub fn secondary(aspect_ratio: f32) -> Self {
        Self {
            fov_y_radians: 60.0_f32.to_radians(),
            aspect_ratio,
            z_near: 0.1,
            z_far: 1000.0,
        }
    }
}

impl<const MAIN: bool> Camera<MAIN> {
    pub fn with_fov_degrees(self, fov_degrees: f32) -> Self {
        Self {
            fov_y_radians: fov_degrees.to_radians(),
            aspect_ratio: self.aspect_ratio,
            z_near: self.z_near,
            z_far: self.z_far,
        }
    }

    pub fn with_clip_planes(self, z_near: f32, z_far: f32) -> Self {
        Self {
            fov_y_radians: self.fov_y_radians,
            aspect_ratio: self.aspect_ratio,
            z_near,
            z_far,
        }
    }

    pub fn projection_matrix(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(self.fov_y_radians, self.aspect_ratio, self.z_near, self.z_far)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect_ratio = width as f32 / height.max(1) as f32;
    }
}
