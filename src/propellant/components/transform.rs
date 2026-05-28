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

    pub fn rotated(self, rotation: glam::Quat) -> Self {
        Self {
            position: self.position,
            rotation,
            scale: self.scale,
        }
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
