use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, Deserialize, Serialize)]
pub struct TubeStation {
    #[serde(rename = "_id")]
    pub id: String, // naptanId
    pub name: String,
    pub coordinates: (f64, f64), // (longitude, latitude)
    pub lines: HashSet<String>,  // tube lines that serve the station
}

impl PartialEq for TubeStation {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TubeStation {}

impl Hash for TubeStation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
