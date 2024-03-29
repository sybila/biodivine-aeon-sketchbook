use crate::sketchbook::{UpdateFn, VarId, Variable};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

/// Structure for sending data about `Variable` and its `UpdateFn` to the frontend.
///
/// `VariableData` contains similar fields as `Variable` and additional fields `id` and `update_fn`.
/// Some fields simplified compared to original typesafe versions (e.g., pure `Strings` are used
/// instead of more complex typesafe structs) to allow for easier (de)serialization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariableData {
    pub id: String,
    pub name: String,
    pub update_fn: String,
}

impl VariableData {
    /// Create new `VariableData` object given a variable's `name` and `id` string slices.
    pub fn new(id: &str, name: &str, update_fn: &str) -> VariableData {
        VariableData {
            id: id.to_string(),
            name: name.to_string(),
            update_fn: update_fn.to_string(),
        }
    }

    /// Create new `VariableData` object given a reference to a variable, its update function,
    /// and its id.
    pub fn from_var(var_id: &VarId, variable: &Variable, update_fn: &UpdateFn) -> VariableData {
        VariableData {
            id: var_id.to_string(),
            name: variable.get_name().to_string(),
            update_fn: update_fn.get_fn_expression().to_string(),
        }
    }
}

impl Display for VariableData {
    /// Use json serialization to convert `VariableData` to string.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl FromStr for VariableData {
    type Err = String;

    /// Use json de-serialization to construct `VariableData` from string.
    fn from_str(s: &str) -> Result<VariableData, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}
