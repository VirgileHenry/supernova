#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        position: glam::Vec3::ZERO,
        rotation: glam::Quat::IDENTITY,
        scale: glam::Vec3::ONE,
    };

    pub fn at(self, position: glam::Vec3) -> Self {
        Self {
            position,
            rotation: self.rotation,
            scale: self.scale,
        }
    }

    /// Update the transform rotation to look at the given point, using Y as up.
    pub fn look_at(self, point: glam::Vec3) -> Self {
        let view_rotation_inv = glam::Quat::look_at_rh(self.position, point, glam::Vec3::Y);
        /* Fixme: need more details here, but since this is a tranform, the look at is made for cams, and we need to inverse it? */
        let view_rotation = view_rotation_inv.inverse();

        Self {
            position: self.position,
            rotation: view_rotation,
            scale: self.scale,
        }
    }

    pub fn rotated(self, rotation: glam::Quat) -> Self {
        Self {
            position: self.position,
            rotation,
            scale: self.scale,
        }
    }

    pub fn rotate_around(&mut self, axis: glam::Vec3, angle: f32) {
        let rotation = glam::Quat::from_axis_angle(axis, angle);
        self.rotation *= rotation;
    }

    pub fn scaled(self, scale: glam::Vec3) -> Self {
        Self {
            position: self.position,
            rotation: self.rotation,
            scale,
        }
    }

    /// Builds the model matrix (TRS) ready to upload to a GPU instance buffer.
    pub fn to_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}
