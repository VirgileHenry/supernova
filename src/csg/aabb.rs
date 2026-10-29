/// An AABB, axis aligned bounding box.
#[derive(Debug, Clone, Copy)]
pub struct AxisAlignedBoundingBox {
    pub(super) min: glam::Vec3,
    pub(super) max: glam::Vec3,
}

impl AxisAlignedBoundingBox {
    pub const ZERO: Self = Self {
        min: glam::Vec3::ZERO,
        max: glam::Vec3::ZERO,
    };

    pub fn min(&self) -> glam::Vec3 {
        self.min
    }

    pub fn max(&self) -> glam::Vec3 {
        self.max
    }

    pub fn union(self, other: Self) -> Self {
        let min = self.min.min(other.min);
        let max = self.max.max(other.max);
        Self { min, max }
    }

    pub fn intersection(self, other: Self) -> Self {
        let min = self.min.max(other.min);
        let max = self.max.min(other.max);
        /* Fixme: this can create negative space bounding boxes */
        Self { min, max }
    }
}
