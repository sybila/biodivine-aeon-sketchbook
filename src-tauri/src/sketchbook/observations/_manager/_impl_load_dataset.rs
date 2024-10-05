use crate::sketchbook::observations::{Dataset, Observation, ObservationManager, VarValue};
use std::fs::File;
use std::str::FromStr;

impl ObservationManager {
    /// Load a dataset from given CSV file. The header line specifies variables, following lines
    /// represent individual observations (id and values).
    ///
    /// For example, the following might be a valid CSV file for a dataset with 2 observations:
    ///    ID,YOX1,CLN3,YHP1,ACE2,SWI5,MBF
    ///    Observation1,0,1,0,1,0,1
    ///    Observation2,1,0,*,1,0,*
    ///
    pub fn load_dataset(name: &str, csv_path: &str) -> Result<Dataset, String> {
        let csv_file = File::open(csv_path).map_err(|e| e.to_string())?;
        let mut rdr = csv::Reader::from_reader(csv_file);

        // parse variable names from the header
        let header = rdr.headers().map_err(|e| e.to_string())?.clone();
        let variables = header.into_iter().skip(1).collect::<Vec<&str>>().clone();

        // parse all raws as observations
        let mut observations = Vec::new();
        for result in rdr.records() {
            let record = result.map_err(|e| e.to_string())?;
            if record.is_empty() {
                return Err("Cannot import empty observation.".to_string());
            }
            let id: &str = record.get(0).unwrap();
            let values: Vec<VarValue> = record
                .iter()
                .skip(1)
                .map(VarValue::from_str)
                .collect::<Result<Vec<VarValue>, String>>()?;
            let observation = Observation::new(values, id)?;
            observations.push(observation);
        }
        Dataset::new(name, observations, variables)
    }

    /// Load a dataset from given CSV file, and add it to this `ObservationManager`. The header
    /// line specifies variables, following lines represent individual observations (id and values).
    pub fn load_and_add_dataset(&mut self, csv_path: &str, id: &str) -> Result<(), String> {
        // use same name as ID
        let dataset = Self::load_dataset(id, csv_path)?;
        self.add_dataset_by_str(id, dataset)
    }
}
