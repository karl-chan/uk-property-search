use crate::lib::{
    math::stats::Stats,
    property::property::{PropertyAction, PropertyStats},
    util::{
        ext::{DecodeJsonResponseExt, VecResultExt},
        globals::Globals,
        http::{Http, HttpOptions},
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
use std::{collections::HashSet, iter};

pub struct Rightmove {
    http: Http,
}

#[derive(Debug, PartialEq)]
pub struct RightmoveProperty {
    id: u32,
    coordinates: (f64, f64), // (longitude, latitude)
    price: u32,              // total (if buy) / monthly (if rent)
    square_feet: Option<i32>,
    post_date: DateTime<Utc>,
    reduced_date: Option<DateTime<Utc>>,
    transacted: bool,
}

impl Rightmove {
    pub fn new(globals: &Globals) -> Rightmove {
        Rightmove {
            http: Http::new(
                globals,
                Some(HttpOptions {
                    max_parallel_connections: Some(
                        globals
                            .properties
                            .get_int("rightmove.max.parallel.connections")
                            as usize,
                    ),
                    max_retry_count: None,
                }),
            ),
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
            property_sub_type: String,
            display_status: String,
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
            frequency: String,
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
            const NUM_PROPERTIES_PER_PAGE: u32 = 24;
            let url = "https://www.rightmove.co.uk/api/_search";
            let query = &[
                ("locationIdentifier", location_identifier),
                ("maxBedrooms", &num_beds.to_string()),
                ("minBedrooms", &num_beds.to_string()),
                (
                    "numberOfPropertiesPerPage",
                    &NUM_PROPERTIES_PER_PAGE.to_string(),
                ),
                ("radius", &radius.to_string()),
                (
                    "index",
                    &(pagination_index * NUM_PROPERTIES_PER_PAGE).to_string(),
                ),
                ("includeSSTC", "true"),
                ("includeLetAgreed", "true"),
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
            // Sometimes rightmove returns 400, so we allow retries.
            let mut remaining_tries = 3;
            loop {
                let result = _self
                    .http
                    .get_with_options(url, query, true)
                    .await?
                    .json_or_err()
                    .await;
                remaining_tries = remaining_tries - 1;

                if result.as_ref().is_ok() || remaining_tries == 0 {
                    return result;
                }
            }
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

        fn parse_price(price: PriceResponse) -> u32 {
            match price.frequency.as_str() {
                "weekly" => price.amount * 52 / 12,
                "yearly" => price.amount / 12,
                "monthly" | "not specified" => price.amount,
                _ => panic!("Unrecognised frequency in price response: {:?}", price),
            }
        }

        fn is_blacklisted(property_response: &PropertyResponse) -> bool {
            lazy_static! {
                static ref BLACKLISTED_PROPERTY_SUBTYPES: HashSet<String> = [
                    "Garages",
                    "Hotel Room",
                    "Land for sale",
                    "Not Specified",
                    "Office",
                    "Parking",
                    "Plot for sale",
                ]
                .into_iter()
                .map(|s| s.to_owned())
                .collect();
            }
            return BLACKLISTED_PROPERTY_SUBTYPES.contains(&property_response.property_sub_type);
        }

        let properties: Vec<RightmoveProperty> = iter::once(response)
            .chain(more_responses.unwrap_all().into_iter())
            .flat_map(|r| r.properties.into_iter())
            .filter(|property| !is_blacklisted(property))
            .sorted_by_key(|property| property.id)
            .dedup_by(|p1, p2| p1.id == p2.id)
            .map(|property| RightmoveProperty {
                id: property.id,
                coordinates: (property.location.longitude, property.location.latitude),
                price: parse_price(property.price),
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
                transacted: property.display_status == "Let agreed"
                    || property.display_status == "Sold STC"
                    || property.display_status == "Under offer",
            })
            .collect();
        Ok(properties)
    }

    pub fn to_stats(&self, properties: Vec<RightmoveProperty>) -> PropertyStats {
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
    use super::{PropertyAction, Rightmove};
    use crate::lib::util::globals::Globals;
    use itertools::Itertools;
    use more_asserts::{assert_gt, assert_lt};

    #[tokio::test]
    async fn test_get_location_identifier() {
        let globals = Globals::new().await;
        let rightmove = Rightmove::new(&globals);
        let location_identifier = rightmove
            .get_location_identifier("N1 9AL".to_owned())
            .await
            .unwrap();
        assert_eq!(location_identifier, "POSTCODE^544984");
    }

    #[tokio::test]
    async fn test_search() {
        let globals = Globals::new().await;
        let rightmove = Rightmove::new(&globals);
        let properties = rightmove
            .search("POSTCODE^544984".to_owned(), PropertyAction::Buy, 2, 0.25)
            .await
            .unwrap();
        assert_gt!(properties.len(), 10);
    }

    #[tokio::test]
    async fn test_search_no_duplicates() {
        let globals = Globals::new().await;
        let rightmove = Rightmove::new(&globals);
        let properties = rightmove
            .search("POSTCODE^544984".to_owned(), PropertyAction::Buy, 2, 0.25)
            .await
            .unwrap();
        assert_eq!(
            properties.iter().map(|p| p.id).sorted().dedup().count(),
            properties.len()
        );
    }

    #[tokio::test]
    async fn test_get_stats() {
        let globals = Globals::new().await;
        let rightmove = Rightmove::new(&globals);
        let properties = rightmove
            .search("POSTCODE^544984".to_owned(), PropertyAction::Buy, 2, 0.25)
            .await
            .unwrap();
        let stats = rightmove.to_stats(properties);
        assert_lt!(stats.price.min, 600_000.0);
        assert_gt!(stats.price.max, 2_000_000.0);
    }
}
