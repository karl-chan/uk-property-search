use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TubeStation {
    pub name: String,
    pub zone: Vec<u8>,
    pub postcode: String,
    pub coordinates: (f64, f64), // (longitude, latitude)
}
