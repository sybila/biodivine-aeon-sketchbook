use crate::sketchbook::{Observability, Regulation, RegulationSign};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Structure for sending simplified data about `Regulation` to frontend.
/// Only contains some fields, in string format, to allow for simpler parsing and manipulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegulationData {
    pub regulator: String,
    pub target: String,
    pub sign: RegulationSign,
    pub observable: Observability,
}

impl RegulationData {
    pub fn new(
        regulator: String,
        target: String,
        observable: Observability,
        sign: RegulationSign,
    ) -> RegulationData {
        RegulationData {
            regulator,
            target,
            observable,
            sign,
        }
    }

    pub fn from_reg(regulation: &Regulation) -> RegulationData {
        RegulationData {
            regulator: regulation.get_regulator().to_string(),
            target: regulation.get_target().to_string(),
            observable: *regulation.get_observability(),
            sign: *regulation.get_sign(),
        }
    }

    pub fn try_from_reg_str(regulation_str: &str) -> Result<RegulationData, String> {
        let regulation = Regulation::try_from_string(regulation_str)?;
        Ok(RegulationData::from_reg(&regulation))
    }
}

impl Display for RegulationData {
    /// Use json serialization to convert `RegulationData` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for RegulationData {
    type Err = String;

    /// Use json de-serialization to construct `RegulationData` from string.
    fn from_str(s: &str) -> Result<RegulationData, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
