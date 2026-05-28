use crate::csg::ops;
use crate::csg::primitive;
use crate::csg::CsgNode;

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
        node: ops::Round,
        child: Box<CsgTree>,
    },
    Extrude {
        node: ops::Extrude,
        child: Box<CsgTree>,
    },
    Revolve {
        node: ops::Revolve,
        child: Box<CsgTree>,
    },
    PlanarSym {
        node: ops::PlanarSymmetry,
        child: Box<CsgTree>,
    },
    AxialSym {
        node: ops::AxialSymmetry,
        child: Box<CsgTree>,
    },
    Elongate {
        node: ops::Elongate,
        child: Box<CsgTree>,
    },
    Onion {
        node: ops::Onion,
        child: Box<CsgTree>,
    },
    Twist {
        node: ops::Twist,
        child: Box<CsgTree>,
    },
    Bend {
        node: ops::Bend,
        child: Box<CsgTree>,
    },
    Transform {
        node: ops::Transform,
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
        node: ops::SmoothUnion,
        children: Vec<CsgTree>,
    },
    SmoothIntersect {
        node: ops::SmoothIntersect,
        children: Vec<CsgTree>,
    },

    // --- Binary asymmetric combinators ---
    Difference {
        base: Box<CsgTree>,
        subtract: Box<CsgTree>,
    },
    SmoothDifference {
        node: ops::SmoothDifference,
        base: Box<CsgTree>,
        subtract: Box<CsgTree>,
    },
}

impl CsgTree {
    pub fn flatten(&self) -> Vec<crate::csg::CsgNodeRepr> {
        let mut nodes = Vec::new();
        self.flatten_into(&mut nodes);
        nodes
    }

    fn flatten_into(&self, out: &mut Vec<crate::csg::CsgNodeRepr>) {
        match self {
            // Empty
            CsgTree::Empty => out.push(crate::csg::CsgNodeRepr::new(crate::csg::opcodes::EMPTY, 0, [0.0; 12])),

            // Primitives push directly.
            CsgTree::Sphere(primitive) => out.push(primitive.to_repr(0)),
            CsgTree::Cube(primitive) => out.push(primitive.to_repr(0)),
            CsgTree::Torus(primitive) => out.push(primitive.to_repr(0)),
            CsgTree::CubeFrame(primitive) => out.push(primitive.to_repr(0)),
            CsgTree::Cone(primitive) => out.push(primitive.to_repr(0)),
            CsgTree::Triangle(primitive) => out.push(primitive.to_repr(0)),
            CsgTree::RegularHexagon(primitive) => out.push(primitive.to_repr(0)),
            CsgTree::Capsule(primitive) => out.push(primitive.to_repr(0)),
            CsgTree::Cylinder(primitive) => out.push(primitive.to_repr(0)),
            CsgTree::Ellipse(primitive) => out.push(primitive.to_repr(0)),
            CsgTree::Octahedron(primitive) => out.push(primitive.to_repr(0)),
            CsgTree::Pyramid(primitive) => out.push(primitive.to_repr(0)),

            // Unary ops: flatten child, then push operator.
            CsgTree::Round { node, child } => {
                out.push(node.to_repr(1));
                child.flatten_into(out);
            }
            CsgTree::Extrude { node, child } => {
                out.push(node.to_repr(1));
                child.flatten_into(out);
            }
            CsgTree::Revolve { node, child } => {
                out.push(node.to_repr(1));
                child.flatten_into(out);
            }
            CsgTree::PlanarSym { node, child } => {
                out.push(node.to_repr(1));
                child.flatten_into(out);
            }
            CsgTree::AxialSym { node, child } => {
                out.push(node.to_repr(1));
                child.flatten_into(out);
            }
            CsgTree::Elongate { node, child } => {
                out.push(node.to_repr(1));
                child.flatten_into(out);
            }
            CsgTree::Onion { node, child } => {
                out.push(node.to_repr(1));
                child.flatten_into(out);
            }
            CsgTree::Twist { node, child } => {
                out.push(node.to_repr(1));
                child.flatten_into(out);
            }
            CsgTree::Bend { node, child } => {
                out.push(node.to_repr(1));
                child.flatten_into(out);
            }
            CsgTree::Transform { node, child } => {
                out.push(node.to_repr(1));
                child.flatten_into(out);
            }

            // N-ary: flatten all children left-to-right, then push op with count.
            CsgTree::Union { children } => {
                out.push(ops::Union.to_repr(children.len() as u32));
                for c in children {
                    c.flatten_into(out);
                }
            }
            CsgTree::Intersect { children } => {
                out.push(ops::Intersect.to_repr(children.len() as u32));
                for c in children {
                    c.flatten_into(out);
                }
            }
            CsgTree::SmoothUnion { node, children } => {
                out.push(node.to_repr(children.len() as u32));
                for c in children {
                    c.flatten_into(out);
                }
            }
            CsgTree::SmoothIntersect { node, children } => {
                out.push(node.to_repr(children.len() as u32));
                for c in children {
                    c.flatten_into(out);
                }
            }

            // Binary asymmetric: base first, then subtract.
            CsgTree::Difference { base, subtract } => {
                out.push(ops::Difference.to_repr(2));
                base.flatten_into(out);
                subtract.flatten_into(out);
            }
            CsgTree::SmoothDifference { node, base, subtract } => {
                out.push(node.to_repr(2));
                base.flatten_into(out);
                subtract.flatten_into(out);
            }
        }
    }
}
