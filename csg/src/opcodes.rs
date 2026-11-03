// Primitives: 0..32
pub const EMPTY: u32 = 0;
pub const SPHERE: u32 = 1;
pub const CUBE: u32 = 2;
pub const TORUS: u32 = 3;
pub const CUBE_FRAME: u32 = 4;
pub const CONE: u32 = 5;
pub const TRIANGLE: u32 = 6;
pub const REGULAR_HEXAGON: u32 = 7;
pub const CAPSULE: u32 = 8;
pub const CYLINDER: u32 = 9;
pub const ELLIPSE: u32 = 10;
pub const OCTAHEDRON: u32 = 11;
pub const PYRAMID: u32 = 12;

// Unary ops: 32..64
pub const ROUND: u32 = 32;
pub const EXTRUDE: u32 = 33;
pub const REVOLVE: u32 = 34;
pub const PLANAR_SYMMETRY: u32 = 35;
pub const AXIAL_SYMMETRY: u32 = 36;
pub const ELONGATE: u32 = 37;
pub const ONION: u32 = 38;
pub const TWIST: u32 = 39;
pub const BEND: u32 = 40;
pub const TRANSFORM: u32 = 41;

// Combinators: 64..
pub const UNION: u32 = 64;
pub const INTERSECT: u32 = 65;
pub const DIFFERENCE: u32 = 66;
pub const SMOOTH_UNION: u32 = 67;
pub const SMOOTH_INTERSECT: u32 = 68;
pub const SMOOTH_DIFFERENCE: u32 = 69;
