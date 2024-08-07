use crate::sketchbook::JsonSerde;
use serde::{Deserialize, Serialize};
use std::time;
use time::Duration;

/// Object encompassing analysis results that are to be send to frontend.
/// It does not contain any intermediate or raw results, these are kept on backend only.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnalysisResults {
    /// Number of satisfying networks.
    num_sat_networks: u64,
    /// Computation time in seconds.
    comp_time: u64,
    /// Any kind of string metadata to be displayed/logged on frontend.
    metadata_log: String,
}

impl<'de> JsonSerde<'de> for AnalysisResults {}

impl AnalysisResults {
    /// Create new `AnalysisState` with a full sketch data.
    pub fn new(num_sat_networks: u64, comp_time: Duration, metadata_log: &str) -> AnalysisResults {
        AnalysisResults {
            num_sat_networks,
            comp_time: comp_time.as_secs(),
            metadata_log: metadata_log.to_string(),
        }
    }

    /// Append string to the end of current metadata.
    pub fn append_metadata(&mut self, new_metadata: &str) {
        self.metadata_log.push_str(new_metadata);
    }
}
