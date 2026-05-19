
#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Round {
        radius: crate::types::Float,
    },
    Extrude {
        extrusion: crate::types::Vec3,
    },
    Revolve {
        axis_base: crate::types::Vec3,
        axis_dir: crate::types::Vec3,
    },
    PlanarSymmetry {
        plane_base: crate::types::Vec3,
        plane_normal: crate::types::Vec3,
    },
    AxialSymmetry {
        axis_base: crate::types::Vec3,
        axis_dir: crate::types::Vec3,
        clone_count: u32,
    },
    Elongate {
        elongation: crate::types::Vec3,
    },
    Onion {
        thickness: crate::types::Float,
    },
    Twist {
        direction: crate::types::Vec3,
        amount: crate::types::Float,
    },
    Bend {
        curvature_normal: crate::types::Vec3,
    },
    Transform {
        offset: crate::types::Vec3,
        size: crate::types::Vec3,
        rotation: crate::types::Quat,
    }
}

impl UnaryOp {
    pub const VAR_COUNT: u32 = 10;

    pub fn id(&self) -> u32 {
        match self {
            UnaryOp::Round { .. } => 0,
            UnaryOp::Extrude { .. } => 1,
            UnaryOp::Revolve { .. } => 2,
            UnaryOp::PlanarSymmetry { .. } => 3,
            UnaryOp::AxialSymmetry { .. } => 4,
            UnaryOp::Elongate { .. } => 5,
            UnaryOp::Onion { .. } => 6,
            UnaryOp::Twist { .. } => 7,
            UnaryOp::Bend { .. } => 8,
            UnaryOp::Transform { .. } => 9,
        }
    }

    pub fn bounding_box(&self, child_bounding_box: crate::types::Float) -> crate::types::Float {
        match self {
            UnaryOp::Round { radius } => child_bounding_box + *radius,
            UnaryOp::Extrude { extrusion } => child_bounding_box + extrusion.length(),
            UnaryOp::Revolve { axis_base, .. } => child_bounding_box + axis_base.length() * 2.0,
            UnaryOp::PlanarSymmetry { plane_base, .. } => child_bounding_box + plane_base.length() * 2.0,
            UnaryOp::AxialSymmetry { axis_base, .. } => child_bounding_box + axis_base.length() * 2.0,
            UnaryOp::Elongate { elongation, .. } => child_bounding_box + elongation.length(),
            UnaryOp::Onion { .. } => child_bounding_box,
            UnaryOp::Twist { .. } => child_bounding_box,
            UnaryOp::Bend { .. } => unimplemented!(),
            UnaryOp::Transform { offset, size, .. } => child_bounding_box * size.length() + offset.length(),
        }
    }

    pub fn pretty_print<W: std::io::Write>(&self, output: &mut W) -> Result<(), std::io::Error> {
        match self {
            UnaryOp::Round { .. } => write!(output, "Round"),
            UnaryOp::Extrude { .. } => write!(output, "Extrude"),
            UnaryOp::Revolve { .. } => write!(output, "Revolve"),
            UnaryOp::PlanarSymmetry { .. } => write!(output, "PlanarSymmetry"),
            UnaryOp::AxialSymmetry { .. } => write!(output, "AxialSymmetry"),
            UnaryOp::Elongate { .. } => write!(output, "Elongate"),
            UnaryOp::Onion { .. } => write!(output, "Onion"),
            UnaryOp::Twist { .. } => write!(output, "Twist"),
            UnaryOp::Bend { .. } => write!(output, "Bend"),
            UnaryOp::Transform { .. } => write!(output, "Transform"),
        }
    }
}