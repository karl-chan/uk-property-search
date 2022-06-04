use crate::lib::{
    math::stats::Stats,
    property::property::{PropertyAction, PropertyStats, PropertyStatsProvider},
    util::{globals::Globals, http::Http},
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::future::join_all;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub struct Rightmove {
    http: Http,
}

#[derive(Debug, PartialEq)]
pub struct RightmoveProperty {
    id: u32,
    coordinates: (f64, f64), // (longitude, latitude)
    price: u32,
    square_feet: Option<i32>,
    post_date: DateTime<Utc>,
    reduced_date: Option<DateTime<Utc>>,
}

impl Rightmove {
    pub fn new(globals: &Globals) -> Rightmove {
        Rightmove {
            http: Http::new(globals, None),
        }
    }

    async fn get_location_identifier(&self, postcode: String) -> Result<String> {
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct LocationIdentifierResponse {
            type_ahead_locations: Vec<TypeAheadLocation>,
        }
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct TypeAheadLocation {
            location_identifier: String,
        }

        let delimited_postcode = postcode
            .chars()
            .chunks(2)
            .into_iter()
            .map(|chunk| chunk.collect::<String>())
            .join("/");
        let url = format!(
            "https://www.rightmove.co.uk/typeAhead/uknostreet/{}",
            &delimited_postcode
        );
        let res: LocationIdentifierResponse = self.http.get(url).await?.json().await?;
        res.type_ahead_locations
            .first()
            .map(|l| l.location_identifier.clone())
            .ok_or(anyhow!(
                "Location identifier not found for postcode: {}!",
                postcode
            ))
    }

    async fn search(
        &self,
        location_identifier: &str,
        action: PropertyAction,
        num_beds: u32,
        radius: f64,
    ) -> Result<Vec<RightmoveProperty>> {
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SearchResponse {
            result_count: String,
            properties: Vec<PropertyResponse>,
            pagination: PaginationResponse,
        }

        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct PropertyResponse {
            id: u32,
            location: LocationResponse,
            price: PriceResponse,
            display_size: String,
            first_visible_date: String,
            listing_update: ListingUpdateResponse,
        }

        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct LocationResponse {
            latitude: f64,
            longitude: f64,
        }

        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct PriceResponse {
            amount: u32,
            currency_code: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ListingUpdateResponse {
            listing_update_reason: String,
            listing_update_date: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct PaginationResponse {
            total: u32,
        }

        async fn search_pagination(
            _self: &Rightmove,
            location_identifier: &str,
            action: PropertyAction,
            num_beds: u32,
            radius: f64,
            pagination_index: u32,
        ) -> Result<SearchResponse> {
            let url = "https://www.rightmove.co.uk/api/_search";
            let query = &[
                ("locationIdentifier", location_identifier),
                ("maxBedrooms", &num_beds.to_string()),
                ("minBedrooms", &num_beds.to_string()),
                ("numberOfPropertiesPerPage", "24"),
                ("radius", &radius.to_string()),
                ("index", &pagination_index.to_string()),
                ("includeSSTC", "true"),
                ("viewType", "LIST"),
                (
                    "channel",
                    match action {
                        PropertyAction::Buy => "BUY",
                        PropertyAction::Rent => "RENT",
                    },
                ),
                ("areaSizeUnit", "sqft"),
                ("currencyCode", "GBP"),
            ];
            let response: SearchResponse = _self
                .http
                .get_with_options(url, query, true)
                .await?
                .json()
                .await?;
            Ok(response)
        }

        let response =
            search_pagination(&self, &location_identifier, action, num_beds, radius, 0).await?;
        let more_responses = join_all(
            (1..response.pagination.total)
                .into_iter()
                .map(|index| {
                    search_pagination(&self, &location_identifier, action, num_beds, radius, index)
                })
                .collect_vec(),
        )
        .await;

        fn parse_square_feet(s: &str) -> Option<i32> {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"(.*) sq. ft.").unwrap();
            }
            RE.captures(s)
                .map(|caps| caps.get(1).unwrap())
                .map(|m| m.as_str().replace(",", "").parse::<i32>().unwrap())
        }

        fn parse_date(s: &str) -> DateTime<Utc> {
            DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
        }

        let properties: Vec<RightmoveProperty> = response
            .properties
            .into_iter()
            .chain(
                more_responses
                    .into_iter()
                    .flat_map(|r| r.unwrap().properties.into_iter()),
            )
            .map(|property| RightmoveProperty {
                id: property.id,
                coordinates: (property.location.longitude, property.location.latitude),
                price: property.price.amount,
                square_feet: parse_square_feet(&property.display_size),
                post_date: parse_date(&property.first_visible_date),
                reduced_date: match property.listing_update.listing_update_reason.as_str() {
                    "price_reduced" => {
                        Some(parse_date(&property.listing_update.listing_update_date))
                    }
                    _ => None,
                },
            })
            .collect();
        Ok(properties)
    }
}

#[async_trait]
impl PropertyStatsProvider for Rightmove {
    async fn get_stats(
        &self,
        postcode: String,
        action: PropertyAction,
        num_beds: u32,
        radius: f64,
    ) -> Result<PropertyStats> {
        let location_identifier = self.get_location_identifier(postcode).await?;
        let properties = self
            .search(&location_identifier, action, num_beds, radius)
            .await?;
        let prices = properties.iter().map(|p| p.price).collect_vec();
        let post_dates_ms = properties
            .iter()
            .map(|p| p.post_date.timestamp_millis() as f64)
            .collect_vec();

        Ok(PropertyStats {
            price: Stats::from_vec(&prices),
            post_date: Stats::from_vec(&post_dates_ms),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{PropertyAction, Rightmove};
    use crate::lib::{property::property::PropertyStatsProvider, util::globals::Globals};
    use more_asserts::{assert_gt, assert_lt};

    #[tokio::test]
    async fn test_get_location_identifier() {
        let globals = Globals::new().await;
        let rightmove = Rightmove::new(&globals);
        let location_identifier = rightmove
            .get_location_identifier("SW1A 2AA".to_owned())
            .await
            .unwrap();
        assert_eq!(location_identifier, "REGION^91989");
    }

    #[tokio::test]
    async fn test_search() {
        let globals = Globals::new().await;
        let rightmove = Rightmove::new(&globals);
        let properties = rightmove
            .search("REGION^91989", PropertyAction::Buy, 2, 0.0)
            .await
            .unwrap();
        assert_gt!(properties.len(), 600);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let globals = Globals::new().await;
        let rightmove = Rightmove::new(&globals);
        let stats = rightmove
            .get_stats("SW1A 2AA".to_owned(), PropertyAction::Buy, 2, 0.0)
            .await
            .unwrap();
        assert_lt!(stats.price.min, 1_000_000.0);
        assert_gt!(stats.price.max, 10_000_000.0);
    }
}
