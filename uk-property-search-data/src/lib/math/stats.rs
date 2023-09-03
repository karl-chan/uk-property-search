use std::f64::NAN;

use serde::{Deserialize, Serialize};
use statrs::statistics::{Data, Max, Median, Min, OrderStatistics};
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Stats {
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
    pub count: usize,
}

impl Stats {
    pub fn from_vec<T: Into<f64> + Copy>(vec: &Vec<T>) -> Stats {
        let mut data: Data<Vec<f64>> = Data::new(vec.iter().map(|v| (*v).into()).collect());
        Stats {
            min: data.min(),
            q1: data.lower_quartile(),
            median: data.median(),
            q3: data.upper_quartile(),
            max: data.max(),
            count: data.len(),
        }
    }

    pub fn nan() -> Stats {
        Stats {
            min: NAN,
            q1: NAN,
            median: NAN,
            q3: NAN,
            max: NAN,
            count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Stats;
    use statrs::assert_almost_eq;

    #[test]
    fn test_from_vec() {
        let v = vec![1, 2, 3, 4];
        let stats = Stats::from_vec(&v);
        assert_eq!(stats.min, 1.0);
        assert_almost_eq!(stats.q1, 1.416666666666666, 1e-15);
        assert_eq!(stats.median, 2.5);
        assert_almost_eq!(stats.q3, 3.583333333333333, 1e-15);
        assert_eq!(stats.max, 4.0);
        assert_eq!(stats.count, 4);
    }
}
