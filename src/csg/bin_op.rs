
#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Union,
    Inter,
    Diff,
    SmoothUnion {
        amount: crate::types::Float,
    },
    SmoothInter {
        amount: crate::types::Float,
    },
    SmoothDiff {
        amount: crate::types::Float,
    }
}

impl BinOp {
    pub const VAR_COUNT: u32 = 6;

    pub fn id(&self) -> u32 {
        match self {
            BinOp::Union => 0,
            BinOp::Inter => 1,
            BinOp::Diff => 2,
            BinOp::SmoothUnion { .. } => 3,
            BinOp::SmoothInter { .. } => 4,
            BinOp::SmoothDiff { .. } => 5,
        }
    }

    pub fn pretty_print<W: std::io::Write>(&self, output: &mut W) -> Result<(), std::io::Error> {
        match self {
            BinOp::Union { .. } => write!(output, "Union"),
            BinOp::Inter { .. } => write!(output, "Inter"),
            BinOp::Diff { .. } => write!(output, "Diff"),
            BinOp::SmoothUnion { .. } => write!(output, "SmoothUnion"),
            BinOp::SmoothInter { .. } => write!(output, "SmoothInter"),
            BinOp::SmoothDiff { .. } => write!(output, "SmoothDiff"),
        }
    }
}