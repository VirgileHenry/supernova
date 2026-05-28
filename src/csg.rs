//* Core idea is to store csg in vecs, contiguous memory
//* We store the csg tree in prefixed notation: so Union Cube Inter Sphere Cube for example

mod opcodes;
mod ops;
mod primitive;
mod repr;
mod tree;
mod types;

pub use repr::CsgNodeRepr;
pub use tree::CsgTree;

pub trait CsgNode {
    const OPCODE: u32;
    fn to_repr(&self, child_count: u32) -> repr::CsgNodeRepr;
}
