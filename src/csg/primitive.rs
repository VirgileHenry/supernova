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
pub struct Sphere {
    pub offset: Vec3,
    pub radius: Float,
}

impl crate::csg::CsgNode for Sphere {
    const OPCODE: u32 = crate::csg::opcodes::SPHERE;
    fn to_repr(&self, _: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.offset);
        p[3] = self.radius;
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, 0, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Cube {
    pub offset: Vec3,
    pub size: Vec3,
    pub rotation: Quat,
}

impl crate::csg::CsgNode for Cube {
    const OPCODE: u32 = crate::csg::opcodes::CUBE;
    fn to_repr(&self, _: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.offset);
        pack_vec3(&mut p, 3, self.size);
        pack_quat(&mut p, 6, self.rotation);
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, 0, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Torus {
    pub offset: Vec3,
    pub normal: Vec3,
    pub inner_radius: Float,
    pub outer_radius: Float,
}

impl crate::csg::CsgNode for Torus {
    const OPCODE: u32 = crate::csg::opcodes::TORUS;
    fn to_repr(&self, _: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.offset);
        pack_vec3(&mut p, 3, self.normal);
        p[6] = self.inner_radius;
        p[7] = self.outer_radius;
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, 0, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CubeFrame {
    pub offset: Vec3,
    pub size: Vec3,
    pub rotation: Quat,
    pub thickness: Float,
}

impl crate::csg::CsgNode for CubeFrame {
    const OPCODE: u32 = crate::csg::opcodes::CUBE_FRAME;
    fn to_repr(&self, _: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.offset);
        pack_vec3(&mut p, 3, self.size);
        // Rotation eats 4 floats but we only have 12 total; pack quat into
        // slots 6..10 and put thickness at 10. Slot 11 unused.
        pack_quat(&mut p, 6, self.rotation);
        p[10] = self.thickness;
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, 0, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Cone {
    pub offset: Vec3,
    pub pointing_towards: Vec3,
    pub height: Float,
    pub base_radius: Float,
}

impl crate::csg::CsgNode for Cone {
    const OPCODE: u32 = crate::csg::opcodes::CONE;
    fn to_repr(&self, _: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.offset);
        pack_vec3(&mut p, 3, self.pointing_towards);
        p[6] = self.height;
        p[7] = self.base_radius;
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, 0, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

impl crate::csg::CsgNode for Triangle {
    const OPCODE: u32 = crate::csg::opcodes::TRIANGLE;
    fn to_repr(&self, _: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.a);
        pack_vec3(&mut p, 3, self.b);
        pack_vec3(&mut p, 6, self.c);
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, 0, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct RegularHexagon {
    pub offset: Vec3,
    pub normal: Vec3,
    pub radius: Float,
}

impl crate::csg::CsgNode for RegularHexagon {
    const OPCODE: u32 = crate::csg::opcodes::REGULAR_HEXAGON;
    fn to_repr(&self, _: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.offset);
        pack_vec3(&mut p, 3, self.normal);
        p[6] = self.radius;
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, 0, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Capsule {
    pub a: Vec3,
    pub b: Vec3,
    pub radius: Float,
}

impl crate::csg::CsgNode for Capsule {
    const OPCODE: u32 = crate::csg::opcodes::CAPSULE;
    fn to_repr(&self, _: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.a);
        pack_vec3(&mut p, 3, self.b);
        p[6] = self.radius;
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, 0, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Cylinder {
    pub a: Vec3,
    pub b: Vec3,
    pub radius: Float,
}

impl crate::csg::CsgNode for Cylinder {
    const OPCODE: u32 = crate::csg::opcodes::CYLINDER;
    fn to_repr(&self, _: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.a);
        pack_vec3(&mut p, 3, self.b);
        p[6] = self.radius;
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, 0, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Ellipse {
    pub a: Vec3,
    pub b: Vec3,
    pub radius: Float,
}

impl crate::csg::CsgNode for Ellipse {
    const OPCODE: u32 = crate::csg::opcodes::ELLIPSE;
    fn to_repr(&self, _: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.a);
        pack_vec3(&mut p, 3, self.b);
        p[6] = self.radius;
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, 0, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Octahedron {
    pub offset: Vec3,
    pub size: Float,
    pub rotation: Quat,
}

impl crate::csg::CsgNode for Octahedron {
    const OPCODE: u32 = crate::csg::opcodes::OCTAHEDRON;
    fn to_repr(&self, _: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.offset);
        p[3] = self.size;
        pack_quat(&mut p, 4, self.rotation);
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, 0, p)
    }
}

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Pyramid {
    pub offset: Vec3,
    pub size: Float,
    pub rotation: Quat,
}

impl crate::csg::CsgNode for Pyramid {
    const OPCODE: u32 = crate::csg::opcodes::PYRAMID;
    fn to_repr(&self, _: u32) -> crate::csg::repr::CsgNodeRepr {
        let mut p = [0.0; 12];
        pack_vec3(&mut p, 0, self.offset);
        p[3] = self.size;
        pack_quat(&mut p, 4, self.rotation);
        crate::csg::repr::CsgNodeRepr::new(Self::OPCODE, 0, p)
    }
}
