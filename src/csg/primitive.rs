
#[derive(Debug, Clone, Copy)]
pub enum Primitive {
    Sphere {
        /// Position of the center of the sphere
        offset: crate::types::Vec3,
        /// Radius of the sphere
        radius: crate::types::Float,
    },
    Cube {
        /// Position of the center of the cube
        offset: crate::types::Vec3,
        /// Size of the cube (width, height, depth)
        size: crate::types::Vec3,
        /// Rotation of the cube
        rotation: glam::Quat,
    },
    Torus {
        /// Position of the center of the torus
        offset: crate::types::Vec3,
        /// Normal of the plane in which is inscribed the coplanar circular axis
        normal: crate::types::Vec3,
        /// Distance between center and torus circle
        inner_radius: crate::types::Float,
        /// Distance between the torus circle and edge
        outter_radius: crate::types::Float,
    },
    CubeFrame {
        /// Position of the center of the frame
        offset: crate::types::Vec3,
        /// Size of the frame (width, height, depth)
        size: crate::types::Vec3,
        /// Rotation of the frame
        rotation: crate::types::Quat,
        /// Thickness of the frame
        thickness: crate::types::Float,
    },
    Cone {
        /// Position of the center of the base
        offset: crate::types::Vec3,
        /// Direction towards the top
        pointing_towards: crate::types::Vec3,
        /// Distance between the base and the top
        height: crate::types::Float,
        /// Radius of the base
        base_radius: crate::types::Float,
    },
    Triangle {
        /// Point A of triangle
        a: crate::types::Vec3,
        /// Point B of triangle
        b: crate::types::Vec3,
        /// Point C of triangle
        c: crate::types::Vec3,
    },
    RegularHexagon {
        /// Center of the hexagon
        offset: crate::types::Vec3,
        /// Normal to the hexagon place
        normal: crate::types::Vec3,
        /// Distance between the center of the hexagon and the corners
        radius: crate::types::Float,
    },
    Capsule {
        /// Center of the first extremity sphere of the capsule
        a: crate::types::Vec3,
        /// Center of the second extremity sphere of the capsule
        b: crate::types::Vec3,
        /// Radius of the capsule
        radius: crate::types::Float,
    },
    Cylinder {
        /// First extremity of the the cylinder
        a: crate::types::Vec3,
        /// Second extremity of the the cylinder
        b: crate::types::Vec3,
        /// Radius of the cylinder
        radius: crate::types::Float,
    },
    Ellipse {
        /// First center of the ellipse
        a: crate::types::Vec3,
        /// Second center of the ellipse
        b: crate::types::Vec3,
        /// Radius of the Ellipse
        radius: crate::types::Float,
    },
    Octahedron {
        /// Center of the Octahedron
        offset: crate::types::Vec3,
        /// Size of the Octahedron
        size: crate::types::Float,
        /// Rotation of the Octahedron
        rotation: crate::types::Quat,
    },
    Pyramid {
        /// Center of the Pyramid
        offset: crate::types::Vec3,
        /// Size of the Pyramid
        size: crate::types::Float,
        /// Rotation of the Pyramid
        rotation: crate::types::Quat,
    },
}

impl Primitive {
    pub(super) const VAR_COUNT: u32 = 12;

    pub(super) fn id(&self) -> u32 {
        match self {
            Primitive::Sphere { .. } => 0,
            Primitive::Cube { .. } => 1,
            Primitive::Torus { .. } => 2,
            Primitive::CubeFrame { .. } => 3,
            Primitive::Cone { .. } => 4,
            Primitive::Triangle { .. } => 5,
            Primitive::RegularHexagon { .. } => 6,
            Primitive::Capsule { .. } => 7,
            Primitive::Cylinder { .. } => 8,
            Primitive::Ellipse { .. } => 9,
            Primitive::Octahedron { .. } => 10,
            Primitive::Pyramid { .. } => 11,
        }
    }

    pub fn sphere(radius: crate::types::Float) -> Primitive {
        Primitive::Sphere { 
            radius,
            offset: crate::types::Vec3::ZERO,
        }
    }

    pub fn cylinder(radius: crate::types::Float) -> Primitive {
        Primitive::Cylinder {
            a: crate::types::Vec3::X,
            b: crate::types::Vec3::NEG_X,
            radius,
        }
    }

    pub fn cube(size: crate::types::Vec3) -> Primitive {
        Primitive::Cube {
            offset: crate::types::Vec3::ZERO,
            size,
            rotation: crate::types::Quat::IDENTITY,
        }
    }

    pub fn at(self, offset: crate::types::Vec3) -> Primitive {
        match self {
            Primitive::Sphere { radius, .. } => Primitive::Sphere { radius, offset },
            Primitive::Cube { size, rotation, .. } => Primitive::Cube { offset, size, rotation },
            Primitive::Cylinder { a, b, radius } => Primitive::Cylinder { a: (a - b) * 0.5 + offset, b: (b - a) * 0.5 + offset, radius },
            // TODO: continue this consrtuctor
            #[allow(unreachable_patterns)]
            _ => panic!("Primitive building error: `at` builder not defined for {self:?}")
        }
    }

    pub fn bounding_box(&self) -> crate::types::Float {
        match self {
            Primitive::Sphere { offset, radius } => offset.length() + radius,
            Primitive::Cube { offset, size, .. } =>
                ((size.x * size.x) + (size.y * size.y) + (size.z * size.z)) + offset.length(),
            Primitive::Torus { inner_radius, outter_radius, offset, .. } => 
                inner_radius + outter_radius + offset.length(),
            Primitive::CubeFrame { offset, size, thickness, .. } =>
                ((size.x * size.x) + (size.y * size.y) + (size.z * size.z)) + offset.length() + (thickness * 0.5),
            Primitive::Cone { offset, height, base_radius, .. } => 
                offset.length() + height.max(*base_radius),
            Primitive::Triangle { a, b, c, } => 
                a.length().max(b.length()).max(c.length()),
            Primitive::RegularHexagon { offset, radius, .. } => 
                offset.length() + radius,
            Primitive::Capsule { a, b, radius } => 
                a.length().max(b.length()) + radius,
            Primitive::Cylinder { a, b, radius } => 
                a.length().max(b.length()) + radius,
            Primitive::Ellipse { a, b, radius } => 
                a.length().max(b.length()) + radius,
            Primitive::Octahedron { offset, size, .. } => 
                offset.length() + size,
            Primitive::Pyramid { offset, size, .. } => 
                offset.length() + size,
        }
    }

    pub fn pretty_print<W: std::io::Write>(&self, output: &mut W) -> Result<(), std::io::Error> {
        match self {
            Primitive::Sphere { .. } => write!(output, "Sphere"),
            Primitive::Cube { .. } => write!(output, "Cube"),
            Primitive::Torus { .. } => write!(output, "Torus"),
            Primitive::CubeFrame { .. } => write!(output, "CubeFrame"),
            Primitive::Cone { .. } => write!(output, "Cone"),
            Primitive::Triangle { .. } => write!(output, "Triangle"),
            Primitive::RegularHexagon { .. } => write!(output, "RegularHexagon"),
            Primitive::Capsule { .. } => write!(output, "Capsule"),
            Primitive::Cylinder { .. } => write!(output, "Cylinder"),
            Primitive::Ellipse { .. } => write!(output, "Ellipse"),
            Primitive::Octahedron { .. } => write!(output, "Octahedron"),
            Primitive::Pyramid { .. } => write!(output, "Pyramid"),
        }
    }
}