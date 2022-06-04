use crate::lib::math::stats::Stats;
use anyhow::Result;
use async_trait::async_trait;
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
}

#[async_trait]
pub trait PropertyStatsProvider {
    async fn get_stats(
        &self,
        postcode: String,
        action: PropertyAction,
        num_beds: u32,
        radius: f64,
    ) -> Result<PropertyStats>;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertySummary {
    pub postcode: String,
    pub coordinates: (f64, f64), // (long, lat)
    pub stats: PropertyStats,
}
