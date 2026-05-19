//! Buildings refer to spaceships, spaceship parts, bases (which are spaceships attached to the ground)

use crate::csg::PackedCsgTrees;


pub struct Segment {
    /// 3 CSGs for the shell, interior and furniture of the segment.
    inner: PackedCsgTrees<(), 3>
}

pub struct Construct {
    /// 3 CSGs for the shell, interior and furniture of the segment.
    inner: PackedCsgTrees<(), 3>,
}


