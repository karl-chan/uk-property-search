use crate::lib::util::{
    ext::DecodeJsonResponseExt,
    globals::Globals,
    http::{Http, HttpOptions},
};
use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::warn;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, iter::once, time::Duration};
use tokio::time::sleep;

pub struct PropertyLog {
    http: Http,
    user: String,
    max_retry_count: u32,
    retry_delay: Duration,
}

#[allow(dead_code)]
impl PropertyLog {
    pub fn new(globals: &Globals) -> PropertyLog {
        PropertyLog {
            http: Http::new(
                globals,
                Some(HttpOptions {
                    max_parallel_connections: Some(
                        globals
                            .properties
                            .get_int("propertylog.max.parallel.connections")
                            as usize,
                    ),
                    max_retry_count: None,
                    referer: Some("https://www.rightmove.co.uk/".to_owned()),
                }),
            ),
            user: globals.properties.get_string("propertylog.user"),
            max_retry_count: globals.properties.get_int("propertylog.max.retry.count") as u32,
            retry_delay: Duration::from_secs(
                globals
                    .properties
                    .get_int("propertylog.retry.delay.seconds") as u64,
            ),
        }
    }

    pub async fn get_history(&self, ids: Vec<u32>) -> Result<Vec<PropertyLogHistory>> {
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct PropertiesResponse {
            properties: HashMap<u32, Property>,
        }

        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Property {
            prices: Vec<Price>,
        }

        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Price {
            date: String, // DD/MM/YYYY
            price: String,
        }

        fn parse_date(date: &str) -> DateTime<Utc> {
            NaiveDate::parse_from_str(date, "%d/%m/%Y")
                .expect(&format!("Failed to parse date: {date}"))
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
        }

        fn parse_price(price: &str) -> Result<u32> {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"(([0-9]+),)*[0-9]+").unwrap();
            }
            RE.captures(&price)
                .context(format!(
                    "Failed to find comma-separated number in price: {price}"
                ))?
                .get(0)
                .unwrap()
                .as_str()
                .replace(",", "")
                .parse::<u32>()
                .context(format!("Failed to parse price as integer: {price}"))
        }

        if ids.is_empty() {
            return Ok(vec![]);
        }

        let form = ids
            .into_iter()
            .enumerate()
            .flat_map(|(i, id)| {
                [
                    (format!("properties[{i}][id]"), id.to_string()),
                    (format!("properties[{i}][price]"), "".to_owned()),
                ]
            })
            .chain(once(("user".to_owned(), self.user.to_owned())))
            .collect_vec();
        let mut retries_left = self.max_retry_count;
        let response = loop {
            let result: Result<PropertiesResponse> = self
                .http
                .post_with_form("https://api.propertylog.net/api/properties", &form)
                .await?
                .json_or_err(&format!("Property log query: [{:?}]", &form))
                .await;
            match result {
                Ok(r) => break r,
                Err(err) => {
                    if retries_left > 0 {
                        warn!(
                            "{}\n{} attempts left, retrying in {} seconds...",
                            err,
                            retries_left,
                            self.retry_delay.as_secs()
                        );
                        retries_left -= 1;
                        sleep(self.retry_delay).await;
                        continue;
                    } else {
                        warn!("Ran out of retries!");
                        return Err(err);
                    }
                }
            }
        };
        let histories = response
            .properties
            .into_iter()
            .map(|(id, property)| {
                let records = property
                    .prices
                    .into_iter()
                    .filter_map(|price| {
                        match (parse_date(&price.date), parse_price(&price.price)) {
                            (date, Ok(p)) => Some(PropertyLogRecord {
                                date: date,
                                price: p,
                            }),
                            (_, Err(cause)) => {
                                warn!("{}", cause);
                                None
                            }
                        }
                    })
                    .sorted_by_key(|record| record.date)
                    .collect_vec();
                PropertyLogHistory { id, records }
            })
            .collect();
        Ok(histories)
    }
}

#[derive(Debug, PartialEq)]
pub struct PropertyLogHistory {
    pub id: u32,
    pub records: Vec<PropertyLogRecord>,
}

#[derive(Debug, PartialEq)]
pub struct PropertyLogRecord {
    pub date: DateTime<Utc>,
    pub price: u32,
}

#[cfg(test)]
mod tests {
    use super::PropertyLog;
    use crate::lib::{
        property::estate_agents::property_log::{PropertyLogHistory, PropertyLogRecord},
        util::globals::Globals,
    };
    use chrono::{TimeZone, Utc};

    #[tokio::test]
    async fn test_get_history_empty() {
        let globals = Globals::new().await;
        let property_log = PropertyLog::new(&globals);
        let history = property_log.get_history(vec![]).await.unwrap();
        assert_eq!(history, vec![])
    }

    #[tokio::test]
    async fn test_get_history() {
        let globals = Globals::new().await;
        let property_log = PropertyLog::new(&globals);
        let history = property_log.get_history(vec![128360372]).await.unwrap();
        assert_eq!(
            history,
            vec![PropertyLogHistory {
                id: 128360372,
                records: vec![
                    PropertyLogRecord {
                        date: Utc.with_ymd_and_hms(2022, 10, 25, 0, 0, 0).unwrap(),
                        price: 17950000
                    },
                    PropertyLogRecord {
                        date: Utc.with_ymd_and_hms(2022, 12, 7, 0, 0, 0).unwrap(),
                        price: 16950000
                    }
                ]
            }]
        )
    }
}
