//* Core idea is to store csg in vecs, contiguous memory
//* We store the csg tree in prefixed notation: so Union Cube Inter Sphere Cube for example

pub mod bin_op;
pub mod node;
pub mod primitive;
pub mod unary_op;


pub use {
    bin_op::BinOp,
    node::CsgNode,
    primitive::Primitive,
    unary_op::UnaryOp,
};

/// This trait is an abstraction over CSG trees.
/// It allows multiple types, or even subpart of types, to be represented as CSGs trees.
pub trait CsgTree<'tree, T: 'tree> {
    fn node_count(&self) -> usize;
    fn height(&self) -> usize;
    fn validate(&self) -> bool;
    fn nodes(&self) -> impl Iterator<Item = &CsgNode>;
    fn payloads(&'tree self) -> impl Iterator<Item = &'tree T>;
    fn pretty_print<W: std::io::Write>(&self, output: &mut W) -> Result<(), std::io::Error>;
    fn bounding_box(&self) -> crate::types::Float;
}

impl<'tree, T> CsgTree<'tree, T> for &'tree [(CsgNode, T)] {
    fn node_count(&self) -> usize {
        match self.get(0) {
            Some((CsgNode::Primitive(_), _)) => 1,
            Some((CsgNode::UnaryOp(_), _)) => 1 + (&self[1..]).node_count(),
            Some((CsgNode::BinOp(_), _)) => {
                let first_child_count = (&self[1..]).node_count();
                let second_child_count = (&self[1+first_child_count..]).node_count();
                1 + first_child_count + second_child_count
            },
            None => 0,
        }
    }
    fn height(&self) -> usize {
        unimplemented!()
    }
    fn validate(&self) -> bool {
        self.node_count() == self.len()
    }
    fn nodes(&self) -> impl Iterator<Item = &CsgNode> {
        self.iter().map(|(node, _)| node)
    }
    fn payloads(&'tree self) -> impl Iterator<Item = &'tree T> {
        self.iter().map(|(_, payload)| payload)
    }
    fn pretty_print<W: std::io::Write>(&self, output: &mut W) -> Result<(), std::io::Error> {
        let mut tree = self.nodes();
        let mut prefix = String::new(); 
        rec_pretty_print(&mut tree, output, &mut prefix)
    }
    fn bounding_box(&self) -> crate::types::Float {
        unimplemented!()
    }
}

fn rec_pretty_print<'t, W: std::io::Write, Tree: Iterator<Item = &'t CsgNode>>(tree: &mut Tree, output: &mut W, prefix: &mut String) -> Result<(), std::io::Error> {
    // I would like to make this iterative and not recursive :(
    match tree.next() {
        Some(CsgNode::Primitive(p)) => {
            p.pretty_print(output)?;
        }
        Some(CsgNode::UnaryOp(u)) => {
            u.pretty_print(output)?;
            write!(output, "\n{prefix}└─")?;
            prefix.push_str("  ");
            rec_pretty_print(tree, output, prefix)?;
            prefix.pop();
            prefix.pop();
        }
        Some(CsgNode::BinOp(b)) => {
            b.pretty_print(output)?;
            write!(output, "\n{prefix}├─")?;
            prefix.push_str("│ ");
            rec_pretty_print(tree, output, prefix)?;
            prefix.pop();
            prefix.pop();
            write!(output, "\n{prefix}└─")?;
            prefix.push_str("  ");
            rec_pretty_print(tree, output, prefix)?;
            prefix.pop();
            prefix.pop();
        }
        None => {/* End of tree */}
    }
    Ok(())
}


pub struct PackedCsgTrees<T, const N: usize> {
    /// This vec must represent N csg trees in prefix notations.
    trees: Vec<(node::CsgNode, T)>,
    /// Offset to access each inner csg tree.
    /// The Ith csg tree is composed of the nodes `trees[offsets[I]..offsets[I+1]]`.
    /// offsets[0] is always 0, except if N is 0, in which case this array is empty.
    offsets: [usize; N],
}


impl<T, const N: usize> PackedCsgTrees<T, N> {
    pub fn ith<const I: usize>(&self) -> impl CsgTree<T> + '_ {
        // const block for comp time assertions
        const {
            if I >= N {
                panic!("Invalid index for const size csg tree array!");
            }
        }
        let lower_bound = self.offsets[I];
        let upper_bound = if I == N - 1 { self.trees.len() } else { I + 1 };
        &self.trees[lower_bound..upper_bound]
    }
}

impl PackedCsgTrees<(), 3> {
    pub fn default_segment() -> PackedCsgTrees<(), 3> {
        PackedCsgTrees {
            trees: vec![
                (CsgNode::Primitive(Primitive::cylinder(0.8)), ()),
                (CsgNode::Primitive(Primitive::cylinder(0.75)), ()),
                (CsgNode::Primitive(Primitive::cube(crate::types::Vec3::new(2.0, 0.03, 0.75))
                    .at(crate::types::Vec3::new(0.0, -0.3, 0.0))), ()),
            ],
            offsets: [
                0,
                1,
                2,
            ]
        }
    }
}
