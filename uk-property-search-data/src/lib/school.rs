use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct School {
    #[serde(rename = "_id")]
    pub id: i64,
    pub name: String,
    pub postcode: String,
    pub coordinates: (f64, f64), // (longitude, latitude)
    pub rating: u8,
    pub inspection_date_ms: Option<i64>, // unix milliseconds
}

#[derive(Copy, Clone, Debug)]
pub enum Rating {
    Unknown = 0,
    Outstanding = 1,
    Good = 2,
    RequiresImprovement = 3,
    Inadequate = 4,
}
