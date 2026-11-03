use crate::types::Float;
use crate::types::Quat;
use crate::types::Vec3;

/// Helper: pack a Vec3 into three slots of the params array.
fn pack_vec3(out: &mut [f32; 12], at: usize, v: Vec3) {
    out[at] = v.x;
    out[at + 1] = v.y;
    out[at + 2] = v.z;
}

/// Helper: pack a quaternion into 4 slots of the params array.
fn pack_quat(out: &mut [f32; 12], at: usize, q: Quat) {
    out[at] = q.x;
    out[at + 1] = q.y;
    out[at + 2] = q.z;
    out[at + 3] = q.w;
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Round {
    pub radius: Float,
}

impl crate::CsgNode for Round {
    const OPCODE: u32 = crate::opcodes::ROUND;
    fn to_repr(&self, child_count: u32) -> crate::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        p[0] = self.radius;
        crate::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
    fn bounding_box(&self) -> crate::AxisAlignedBoundingBox {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Extrude {
    pub extrusion: Vec3,
}

impl crate::CsgNode for Extrude {
    const OPCODE: u32 = crate::opcodes::EXTRUDE;
    fn to_repr(&self, child_count: u32) -> crate::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.extrusion);
        crate::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
    fn bounding_box(&self) -> crate::AxisAlignedBoundingBox {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Revolve {
    pub axis_base: Vec3,
    pub axis_dir: Vec3,
}

impl crate::CsgNode for Revolve {
    const OPCODE: u32 = crate::opcodes::REVOLVE;
    fn to_repr(&self, child_count: u32) -> crate::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.axis_base); // p[0..3]
        pack_vec3(&mut p, 4, self.axis_dir); // p[4..7]
        crate::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
    fn bounding_box(&self) -> crate::AxisAlignedBoundingBox {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PlanarSymmetry {
    pub plane_base: Vec3,
    pub plane_normal: Vec3,
}

impl crate::CsgNode for PlanarSymmetry {
    const OPCODE: u32 = crate::opcodes::PLANAR_SYMMETRY;
    fn to_repr(&self, child_count: u32) -> crate::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.plane_base);
        pack_vec3(&mut p, 4, self.plane_normal);
        crate::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
    fn bounding_box(&self) -> crate::AxisAlignedBoundingBox {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AxialSymmetry {
    pub axis_base: Vec3,
    pub axis_dir: Vec3,
    pub clone_count: u32,
}

impl crate::CsgNode for AxialSymmetry {
    const OPCODE: u32 = crate::opcodes::AXIAL_SYMMETRY;
    fn to_repr(&self, child_count: u32) -> crate::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.axis_base);
        pack_vec3(&mut p, 4, self.axis_dir);
        // clone_count is u32 but the buffer is [f32; 12].
        // Bitcast through a union or transmute. f32::from_bits is the clean way.
        p[8] = f32::from_bits(self.clone_count);
        crate::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
    fn bounding_box(&self) -> crate::AxisAlignedBoundingBox {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Elongate {
    pub elongation: Vec3,
}

impl crate::CsgNode for Elongate {
    const OPCODE: u32 = crate::opcodes::ELONGATE;
    fn to_repr(&self, child_count: u32) -> crate::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.elongation);
        crate::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
    fn bounding_box(&self) -> crate::AxisAlignedBoundingBox {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Onion {
    pub thickness: Float,
}

impl crate::CsgNode for Onion {
    const OPCODE: u32 = crate::opcodes::ONION;
    fn to_repr(&self, child_count: u32) -> crate::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        p[0] = self.thickness;
        crate::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
    fn bounding_box(&self) -> crate::AxisAlignedBoundingBox {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Twist {
    pub direction: Vec3,
    pub amount: Float,
}

impl crate::CsgNode for Twist {
    const OPCODE: u32 = crate::opcodes::TWIST;
    fn to_repr(&self, child_count: u32) -> crate::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.direction);
        p[3] = self.amount; // packed into the 4th slot of the direction vec
        crate::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
    fn bounding_box(&self) -> crate::AxisAlignedBoundingBox {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Bend {
    pub curvature_normal: Vec3,
}

impl crate::CsgNode for Bend {
    const OPCODE: u32 = crate::opcodes::BEND;
    fn to_repr(&self, child_count: u32) -> crate::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.curvature_normal);
        crate::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
    fn bounding_box(&self) -> crate::AxisAlignedBoundingBox {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Transform {
    pub offset: Vec3,
    pub size: Vec3,
    pub rotation: Quat,
}

impl crate::CsgNode for Transform {
    const OPCODE: u32 = crate::opcodes::TRANSFORM;
    fn to_repr(&self, child_count: u32) -> crate::repr::CsgNodeRepr {
        // offset (3) + size (3) + quat (4) = 10 floats. Fits in 12 with 2 to spare.
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.offset); // p[0..3]
        pack_vec3(&mut p, 3, self.size); // p[3..6]
        pack_quat(&mut p, 6, self.rotation); // p[6..10]
        crate::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
    fn bounding_box(&self) -> crate::AxisAlignedBoundingBox {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Union;
impl crate::CsgNode for Union {
    const OPCODE: u32 = crate::opcodes::UNION;
    fn to_repr(&self, child_count: u32) -> crate::repr::CsgNodeRepr {
        crate::repr::CsgNodeRepr::new(Self::OPCODE, child_count, [0.0; 12])
    }
    fn bounding_box(&self) -> crate::AxisAlignedBoundingBox {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Intersect;
impl crate::CsgNode for Intersect {
    const OPCODE: u32 = crate::opcodes::INTERSECT;
    fn to_repr(&self, child_count: u32) -> crate::repr::CsgNodeRepr {
        crate::repr::CsgNodeRepr::new(Self::OPCODE, child_count, [0.0; 12])
    }
    fn bounding_box(&self) -> crate::AxisAlignedBoundingBox {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SmoothUnion {
    pub amount: Float,
}

impl crate::CsgNode for SmoothUnion {
    const OPCODE: u32 = crate::opcodes::SMOOTH_UNION;
    fn to_repr(&self, child_count: u32) -> crate::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        p[0] = self.amount;
        crate::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
    fn bounding_box(&self) -> crate::AxisAlignedBoundingBox {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SmoothIntersect {
    pub amount: Float,
}

impl crate::CsgNode for SmoothIntersect {
    const OPCODE: u32 = crate::opcodes::SMOOTH_INTERSECT;
    fn to_repr(&self, child_count: u32) -> crate::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        p[0] = self.amount;
        crate::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
    fn bounding_box(&self) -> crate::AxisAlignedBoundingBox {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Difference;
impl crate::CsgNode for Difference {
    const OPCODE: u32 = crate::opcodes::DIFFERENCE;
    fn to_repr(&self, child_count: u32) -> crate::repr::CsgNodeRepr {
        // child_count is always 2 for Difference, but pass it through anyway
        // for consistency with the trait.
        crate::repr::CsgNodeRepr::new(Self::OPCODE, child_count, [0.0; 12])
    }
    fn bounding_box(&self) -> crate::AxisAlignedBoundingBox {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SmoothDifference {
    pub amount: Float,
}

impl crate::CsgNode for SmoothDifference {
    const OPCODE: u32 = crate::opcodes::SMOOTH_DIFFERENCE;
    fn to_repr(&self, child_count: u32) -> crate::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        p[0] = self.amount;
        crate::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
    fn bounding_box(&self) -> crate::AxisAlignedBoundingBox {
        unimplemented!()
    }
}
