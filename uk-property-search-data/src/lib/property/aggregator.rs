use crate::lib::math::stats::Stats;

use super::{estate_agents::rightmove::RightmoveProperty, property::PropertyStats};
use chrono::Utc;
use itertools::Itertools;

pub struct PropertyAggregator {}

impl PropertyAggregator {
    pub fn calculate_stats(properties: Vec<RightmoveProperty>) -> PropertyStats {
        let percent_transacted_value = if properties.is_empty() {
            0f64
        } else {
            (properties.iter().filter(|p| p.transacted).count() as f64) / (properties.len() as f64)
        };

        let prices = properties.iter().map(|p| p.price).collect_vec();
        let listed_days = properties
            .iter()
            .map(|p| (Utc::now() - p.post_date).num_days() as f64)
            .collect_vec();
        let percent_transacted = properties
            .iter()
            .map(|_| percent_transacted_value)
            .collect_vec();
        let square_feet = properties
            .iter()
            .filter_map(|p| p.square_feet)
            .collect_vec();

        PropertyStats {
            price: Stats::from_vec(&prices),
            listed_days: Stats::from_vec(&listed_days),
            percent_transacted: Stats::from_vec(&percent_transacted),
            square_feet: Stats::from_vec(&square_feet),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lib::{
        math::stats::Stats,
        property::{aggregator::PropertyAggregator, estate_agents::rightmove::RightmoveProperty},
    };
    use chrono::{TimeZone, Utc};

    #[tokio::test]
    async fn test_get_stats() {
        let properties = vec![
            RightmoveProperty {
                id: 105233438,
                coordinates: (-0.122191, 51.53419),
                price: 3600000,
                square_feet: None,
                post_date: Utc.with_ymd_and_hms(2021, 4, 8, 19, 28, 38).unwrap(),
                reduced_date: None,
                transacted: false,
            },
            RightmoveProperty {
                id: 136850450,
                coordinates: (-0.125412, 51.529891),
                price: 3550000,
                square_feet: None,
                post_date: Utc.with_ymd_and_hms(2023, 7, 3, 0, 33, 55).unwrap(),
                reduced_date: None,
                transacted: false,
            },
            RightmoveProperty {
                id: 131749937,
                coordinates: (-0.125412, 51.529891),
                price: 1500000,
                square_feet: None,
                post_date: Utc.with_ymd_and_hms(2023, 2, 15, 21, 9, 3).unwrap(),
                reduced_date: None,
                transacted: false,
            },
        ];

        let stats = PropertyAggregator::calculate_stats(properties);

        assert_eq!(
            stats.price,
            Stats {
                min: 1500000.0,
                q1: 1841666.6666666667,
                median: 3550000.0,
                q3: 3591666.6666666665,
                max: 3600000.0,
                count: 3
            }
        );
        assert_eq!(
            stats.percent_transacted,
            Stats {
                min: 0.0,
                q1: 0.0,
                median: 0.0,
                q3: 0.0,
                max: 0.0,
                count: 3
            }
        );
    }
}
