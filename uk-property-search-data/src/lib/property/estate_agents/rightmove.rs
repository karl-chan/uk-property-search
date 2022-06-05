use crate::lib::{
    math::stats::Stats,
    property::property::{PropertyAction, PropertyStats},
    util::{
        ext::{DecodeJsonResponseExt, VecResultExt},
        globals::Globals,
        http::Http,
    },
};
use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use futures::future::join_all;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::iter;

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

    pub async fn get_location_identifier(&self, postcode: String) -> Result<String> {
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

        let url = format!(
            "https://www.rightmove.co.uk/property-for-sale/search.html?searchLocation={}",
            &postcode
        );
        let html = self.http.get(url).await?.text().await?;
        lazy_static! {
            static ref SELECTOR: Selector = Selector::parse("#locationIdentifier").unwrap();
        }
        match Html::parse_document(&html)
            .select(&SELECTOR)
            .next()
            .unwrap()
            .value()
            .attr("value")
            .filter(|location_identifier| !location_identifier.is_empty())
            .map(|location_identifier| location_identifier.to_owned())
        {
            Some(location_identifier) => Ok(location_identifier),
            None => bail!(
                "Location identifier not found for postcode: [{}]!",
                postcode
            ),
        }
    }

    pub async fn search(
        &self,
        location_identifier: String,
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
            display_size: Option<String>,
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
            listing_update_reason: Option<String>,
            listing_update_date: Option<String>,
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
                .json_or_err()
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

        fn parse_square_feet(maybe_display_size: Option<String>) -> Option<i32> {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"(.*) sq. ft.").unwrap();
            }
            maybe_display_size.and_then(|display_size| {
                RE.captures(&display_size)
                    .map(|caps| caps.get(1).unwrap())
                    .map(|m| m.as_str().replace(",", "").parse::<i32>().unwrap())
            })
        }

        fn parse_date(s: &str) -> DateTime<Utc> {
            DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
        }

        let properties: Vec<RightmoveProperty> = iter::once(response)
            .chain(more_responses.unwrap_all().into_iter())
            .flat_map(|r| r.properties.into_iter())
            .map(|property| RightmoveProperty {
                id: property.id,
                coordinates: (property.location.longitude, property.location.latitude),
                price: property.price.amount,
                square_feet: parse_square_feet(property.display_size),
                post_date: parse_date(&property.first_visible_date),
                reduced_date: property
                    .listing_update
                    .listing_update_reason
                    .and_then(|reason| match reason.as_str() {
                        "price_reduced" => property
                            .listing_update
                            .listing_update_date
                            .map(|date| parse_date(&date)),
                        _ => None,
                    }),
            })
            .collect();
        Ok(properties)
    }

    pub fn to_stats(&self, properties: Vec<RightmoveProperty>) -> PropertyStats {
        let prices = properties.iter().map(|p| p.price).collect_vec();
        let post_dates_ms = properties
            .iter()
            .map(|p| p.post_date.timestamp_millis() as f64)
            .collect_vec();

        PropertyStats {
            price: Stats::from_vec(&prices),
            post_date: Stats::from_vec(&post_dates_ms),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PropertyAction, Rightmove};
    use crate::lib::util::globals::Globals;
    use more_asserts::{assert_gt, assert_lt};

    #[tokio::test]
    async fn test_get_location_identifier() {
        let globals = Globals::new().await;
        let rightmove = Rightmove::new(&globals);
        let location_identifier = rightmove
            .get_location_identifier("SW1A 2AA".to_owned())
            .await
            .unwrap();
        assert_eq!(location_identifier, "POSTCODE^1246000");
    }

    #[tokio::test]
    async fn test_search() {
        let globals = Globals::new().await;
        let rightmove = Rightmove::new(&globals);
        let properties = rightmove
            .search("POSTCODE^1246000".to_owned(), PropertyAction::Buy, 2, 0.25)
            .await
            .unwrap();
        assert_gt!(properties.len(), 10);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let globals = Globals::new().await;
        let rightmove = Rightmove::new(&globals);
        let properties = rightmove
            .search("POSTCODE^1246000".to_owned(), PropertyAction::Buy, 2, 0.25)
            .await
            .unwrap();
        let stats = rightmove.to_stats(properties);
        assert_lt!(stats.price.min, 1_000_000.0);
        assert_gt!(stats.price.max, 5_000_000.0);
    }
}
