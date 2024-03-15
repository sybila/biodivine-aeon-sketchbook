use biodivine_lib_param_bn::BinaryOp as OtherBinaryOp;
use core::convert::From;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

/// Possible binary Boolean operators that can appear in `FnUpdate`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    And,
    Or,
    Xor,
    Iff,
    Imp,
}

impl From<OtherBinaryOp> for BinaryOp {
    fn from(value: OtherBinaryOp) -> Self {
        match value {
            OtherBinaryOp::And => BinaryOp::And,
            OtherBinaryOp::Or => BinaryOp::Or,
            OtherBinaryOp::Xor => BinaryOp::Xor,
            OtherBinaryOp::Iff => BinaryOp::Iff,
            OtherBinaryOp::Imp => BinaryOp::Imp,
        }
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let symbol = match self {
            BinaryOp::And => "&",
            BinaryOp::Or => "|",
            BinaryOp::Xor => "^",
            BinaryOp::Imp => "=>",
            BinaryOp::Iff => "<=>",
        };
        write!(f, "{}", symbol)
    }
}

impl BinaryOp {
    pub fn to_lib_param_bn_version(&self) -> OtherBinaryOp {
        match self {
            BinaryOp::And => OtherBinaryOp::And,
            BinaryOp::Or => OtherBinaryOp::Or,
            BinaryOp::Xor => OtherBinaryOp::Xor,
            BinaryOp::Iff => OtherBinaryOp::Iff,
            BinaryOp::Imp => OtherBinaryOp::Imp,
        }
    }
}
