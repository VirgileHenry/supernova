use crate::csg::primitive;
use crate::csg::types::Float;
use crate::csg::types::Quat;
use crate::csg::types::Vec3;

/// Authored CSG tree. Each variant either is a primitive leaf, a unary op
/// with a boxed child, an n-ary combinator with a vec of children, or a
/// binary asymmetric op (difference) with named base/subtract children.
///
/// Loaded from .ron via serde. Convert to `FlatCsg` with [`Self::flatten`]
/// before GPU upload.
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum CsgTree {
    /// Empty CSG tree
    Empty,

    // --- Primitives ---
    Sphere(primitive::Sphere),
    Cube(primitive::Cube),
    Torus(primitive::Torus),
    CubeFrame(primitive::CubeFrame),
    Cone(primitive::Cone),
    Triangle(primitive::Triangle),
    RegularHexagon(primitive::RegularHexagon),
    Capsule(primitive::Capsule),
    Cylinder(primitive::Cylinder),
    Ellipse(primitive::Ellipse),
    Octahedron(primitive::Octahedron),
    Pyramid(primitive::Pyramid),

    // --- Unary ops (one inline child) ---
    Round {
        radius: Float,
        child: Box<CsgTree>,
    },
    Extrude {
        extrusion: Vec3,
        child: Box<CsgTree>,
    },
    Revolve {
        axis_base: Vec3,
        axis_dir: Vec3,
        child: Box<CsgTree>,
    },
    PlanarSym {
        plane_base: Vec3,
        plane_normal: Vec3,
        child: Box<CsgTree>,
    },
    AxialSym {
        axis_base: Vec3,
        axis_dir: Vec3,
        clone_count: u32,
        child: Box<CsgTree>,
    },
    Elongate {
        elongation: Vec3,
        child: Box<CsgTree>,
    },
    Onion {
        thickness: Float,
        child: Box<CsgTree>,
    },
    Twist {
        direction: Vec3,
        amount: Float,
        child: Box<CsgTree>,
    },
    Bend {
        curvature_normal: Vec3,
        child: Box<CsgTree>,
    },
    Transform {
        offset: Vec3,
        size: Vec3,
        rotation: Quat,
        child: Box<CsgTree>,
    },

    // --- N-ary combinators ---
    Union {
        children: Vec<CsgTree>,
    },
    Intersect {
        children: Vec<CsgTree>,
    },
    SmoothUnion {
        amount: Float,
        children: Vec<CsgTree>,
    },
    SmoothIntersect {
        amount: Float,
        children: Vec<CsgTree>,
    },

    // --- Binary asymmetric combinators ---
    Difference {
        base: Box<CsgTree>,
        subtract: Box<CsgTree>,
    },
    SmoothDifference {
        amount: Float,
        base: Box<CsgTree>,
        subtract: Box<CsgTree>,
    },
}

/*
impl CsgTree {
    /// Flatten to post-order. Operands come before operators; n-ary ops
    /// carry their child count so the GPU stack-evaluator knows how many
    /// values to pop.
    pub fn flatten(&self) -> FlatCsg {
        let mut nodes = Vec::new();
        self.flatten_into(&mut nodes);
        FlatCsg { nodes }
    }

    fn flatten_into(&self, out: &mut Vec<FlatNode>) {
        match self {
            // Primitives push directly.
            CsgTree::Sphere(s) => out.push(FlatNode::Sphere(*s)),
            CsgTree::Cube(c) => out.push(FlatNode::Cube(*c)),
            CsgTree::Torus(t) => out.push(FlatNode::Torus(*t)),
            CsgTree::CubeFrame(c) => out.push(FlatNode::CubeFrame(*c)),
            CsgTree::Cone(c) => out.push(FlatNode::Cone(*c)),
            CsgTree::Triangle(t) => out.push(FlatNode::Triangle(*t)),
            CsgTree::RegularHexagon(h) => out.push(FlatNode::RegularHexagon(*h)),
            CsgTree::Capsule(c) => out.push(FlatNode::Capsule(*c)),
            CsgTree::Cylinder(c) => out.push(FlatNode::Cylinder(*c)),
            CsgTree::Ellipse(e) => out.push(FlatNode::Ellipse(*e)),
            CsgTree::Octahedron(o) => out.push(FlatNode::Octahedron(*o)),
            CsgTree::Pyramid(p) => out.push(FlatNode::Pyramid(*p)),

            // Unary ops: flatten child, then push operator.
            CsgTree::Round { radius, child } => {
                child.flatten_into(out);
                out.push(FlatNode::Round { radius: *radius });
            }
            CsgTree::Extrude { extrusion, child } => {
                child.flatten_into(out);
                out.push(FlatNode::Extrude { extrusion: *extrusion });
            }
            CsgTree::Revolve {
                axis_base,
                axis_dir,
                child,
            } => {
                child.flatten_into(out);
                out.push(FlatNode::Revolve {
                    axis_base: *axis_base,
                    axis_dir: *axis_dir,
                });
            }
            CsgTree::PlanarSym {
                plane_base,
                plane_normal,
                child,
            } => {
                child.flatten_into(out);
                out.push(FlatNode::PlanarSym {
                    plane_base: *plane_base,
                    plane_normal: *plane_normal,
                });
            }
            CsgTree::AxialSym {
                axis_base,
                axis_dir,
                clone_count,
                child,
            } => {
                child.flatten_into(out);
                out.push(FlatNode::AxialSym {
                    axis_base: *axis_base,
                    axis_dir: *axis_dir,
                    clone_count: *clone_count,
                });
            }
            CsgTree::Elongate { elongation, child } => {
                child.flatten_into(out);
                out.push(FlatNode::Elongate { elongation: *elongation });
            }
            CsgTree::Onion { thickness, child } => {
                child.flatten_into(out);
                out.push(FlatNode::Onion { thickness: *thickness });
            }
            CsgTree::Twist {
                direction,
                amount,
                child,
            } => {
                child.flatten_into(out);
                out.push(FlatNode::Twist {
                    direction: *direction,
                    amount: *amount,
                });
            }
            CsgTree::Bend { curvature_normal, child } => {
                child.flatten_into(out);
                out.push(FlatNode::Bend {
                    curvature_normal: *curvature_normal,
                });
            }
            CsgTree::Transform {
                offset,
                size,
                rotation,
                child,
            } => {
                child.flatten_into(out);
                out.push(FlatNode::Transform {
                    offset: *offset,
                    size: *size,
                    rotation: *rotation,
                });
            }

            // N-ary: flatten all children left-to-right, then push op with count.
            CsgTree::Union { children } => {
                for c in children {
                    c.flatten_into(out);
                }
                out.push(FlatNode::Union {
                    child_count: children.len() as u32,
                });
            }
            CsgTree::Intersect { children } => {
                for c in children {
                    c.flatten_into(out);
                }
                out.push(FlatNode::Intersect {
                    child_count: children.len() as u32,
                });
            }
            CsgTree::SmoothUnion { amount, children } => {
                for c in children {
                    c.flatten_into(out);
                }
                out.push(FlatNode::SmoothUnion {
                    amount: *amount,
                    child_count: children.len() as u32,
                });
            }
            CsgTree::SmoothIntersect { amount, children } => {
                for c in children {
                    c.flatten_into(out);
                }
                out.push(FlatNode::SmoothIntersect {
                    amount: *amount,
                    child_count: children.len() as u32,
                });
            }

            // Binary asymmetric: base first, then subtract.
            CsgTree::Difference { base, subtract } => {
                base.flatten_into(out);
                subtract.flatten_into(out);
                out.push(FlatNode::Difference);
            }
            CsgTree::SmoothDifference { amount, base, subtract } => {
                base.flatten_into(out);
                subtract.flatten_into(out);
                out.push(FlatNode::SmoothDifference { amount: *amount });
            }
        }
    }
}
 */
