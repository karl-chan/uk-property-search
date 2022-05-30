use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct School {
    pub id: i64,
    pub name: String,
    pub postcode: String,
    pub coordinates: (f64, f64),
    pub rating: u8,
    pub inspection_date: Option<i64>, // unix milliseconds
}

pub enum Rating {
    Unknown = 0,
    Outstanding = 1,
    Good = 2,
    RequiresImprovement = 3,
    Inadequate = 4,
}
