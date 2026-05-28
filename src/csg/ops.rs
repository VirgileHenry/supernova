use crate::csg::types::Float;
use crate::csg::types::Quat;
use crate::csg::types::Vec3;

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

impl crate::csg::CsgNode for Round {
    const OPCODE: u32 = crate::csg::opcodes::ROUND;
    fn to_repr(&self, child_count: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        p[0] = self.radius;
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Extrude {
    pub extrusion: Vec3,
}

impl crate::csg::CsgNode for Extrude {
    const OPCODE: u32 = crate::csg::opcodes::EXTRUDE;
    fn to_repr(&self, child_count: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.extrusion);
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Revolve {
    pub axis_base: Vec3,
    pub axis_dir: Vec3,
}

impl crate::csg::CsgNode for Revolve {
    const OPCODE: u32 = crate::csg::opcodes::REVOLVE;
    fn to_repr(&self, child_count: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.axis_base); // p[0..3]
        pack_vec3(&mut p, 4, self.axis_dir); // p[4..7]
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PlanarSymmetry {
    pub plane_base: Vec3,
    pub plane_normal: Vec3,
}

impl crate::csg::CsgNode for PlanarSymmetry {
    const OPCODE: u32 = crate::csg::opcodes::PLANAR_SYMMETRY;
    fn to_repr(&self, child_count: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.plane_base);
        pack_vec3(&mut p, 4, self.plane_normal);
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AxialSymmetry {
    pub axis_base: Vec3,
    pub axis_dir: Vec3,
    pub clone_count: u32,
}

impl crate::csg::CsgNode for AxialSymmetry {
    const OPCODE: u32 = crate::csg::opcodes::AXIAL_SYMMETRY;
    fn to_repr(&self, child_count: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.axis_base);
        pack_vec3(&mut p, 4, self.axis_dir);
        // clone_count is u32 but the buffer is [f32; 12].
        // Bitcast through a union or transmute. f32::from_bits is the clean way.
        p[8] = f32::from_bits(self.clone_count);
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Elongate {
    pub elongation: Vec3,
}

impl crate::csg::CsgNode for Elongate {
    const OPCODE: u32 = crate::csg::opcodes::ELONGATE;
    fn to_repr(&self, child_count: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.elongation);
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Onion {
    pub thickness: Float,
}

impl crate::csg::CsgNode for Onion {
    const OPCODE: u32 = crate::csg::opcodes::ONION;
    fn to_repr(&self, child_count: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        p[0] = self.thickness;
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Twist {
    pub direction: Vec3,
    pub amount: Float,
}

impl crate::csg::CsgNode for Twist {
    const OPCODE: u32 = crate::csg::opcodes::TWIST;
    fn to_repr(&self, child_count: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.direction);
        p[3] = self.amount; // packed into the 4th slot of the direction vec
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Bend {
    pub curvature_normal: Vec3,
}

impl crate::csg::CsgNode for Bend {
    const OPCODE: u32 = crate::csg::opcodes::BEND;
    fn to_repr(&self, child_count: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.curvature_normal);
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Transform {
    pub offset: Vec3,
    pub size: Vec3,
    pub rotation: Quat,
}

impl crate::csg::CsgNode for Transform {
    const OPCODE: u32 = crate::csg::opcodes::TRANSFORM;
    fn to_repr(&self, child_count: u32) -> crate::csg::repr::CsgNodeRepr {
        // offset (3) + size (3) + quat (4) = 10 floats. Fits in 12 with 2 to spare.
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.offset); // p[0..3]
        pack_vec3(&mut p, 3, self.size); // p[3..6]
        pack_quat(&mut p, 6, self.rotation); // p[6..10]
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Union;
impl crate::csg::CsgNode for Union {
    const OPCODE: u32 = crate::csg::opcodes::UNION;
    fn to_repr(&self, child_count: u32) -> crate::csg::repr::CsgNodeRepr {
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, child_count, [0.0; 12])
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Intersect;
impl crate::csg::CsgNode for Intersect {
    const OPCODE: u32 = crate::csg::opcodes::INTERSECT;
    fn to_repr(&self, child_count: u32) -> crate::csg::repr::CsgNodeRepr {
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, child_count, [0.0; 12])
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SmoothUnion {
    pub amount: Float,
}

impl crate::csg::CsgNode for SmoothUnion {
    const OPCODE: u32 = crate::csg::opcodes::SMOOTH_UNION;
    fn to_repr(&self, child_count: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        p[0] = self.amount;
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SmoothIntersect {
    pub amount: Float,
}

impl crate::csg::CsgNode for SmoothIntersect {
    const OPCODE: u32 = crate::csg::opcodes::SMOOTH_INTERSECT;
    fn to_repr(&self, child_count: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        p[0] = self.amount;
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Difference;
impl crate::csg::CsgNode for Difference {
    const OPCODE: u32 = crate::csg::opcodes::DIFFERENCE;
    fn to_repr(&self, child_count: u32) -> crate::csg::repr::CsgNodeRepr {
        // child_count is always 2 for Difference, but pass it through anyway
        // for consistency with the trait.
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, child_count, [0.0; 12])
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SmoothDifference {
    pub amount: Float,
}

impl crate::csg::CsgNode for SmoothDifference {
    const OPCODE: u32 = crate::csg::opcodes::SMOOTH_DIFFERENCE;
    fn to_repr(&self, child_count: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        p[0] = self.amount;
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, child_count, p)
    }
}
