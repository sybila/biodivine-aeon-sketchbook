use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InferenceType {
    FullInference,
    StaticInference,
    DynamicInference,
}