use crate::lib::math::stats::Stats;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug)]
pub enum PropertyAction {
    Buy,
    Rent,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PropertyStats {
    pub price: Stats,
    pub post_date: Stats,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertySummary {
    pub postcode: String,
    pub coordinates: (f64, f64), // (long, lat)
    pub stats: PropertyStats,
}
