use crate::lib::math::stats::Stats;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug)]
pub enum PropertyAction {
    Buy = 1,
    Rent = 2,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PropertyStats {
    pub price: Stats,
    pub days: Stats, // how long the advert has been on the market
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertySummary {
    pub postcode: String,
    pub coordinates: (f64, f64), // (long, lat)
    pub action: u8,
    pub num_beds: u32,
    pub stats: PropertyStats,
}
