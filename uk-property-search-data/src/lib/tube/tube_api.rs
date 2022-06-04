use super::tube::TubeStation;
use crate::lib::util::{globals::Globals, http::Http};
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub struct TubeApi {
    http: Http,
}

impl TubeApi {
    pub fn new(globals: &Globals) -> TubeApi {
        TubeApi {
            http: Http::new(&globals, None),
        }
    }

    pub async fn get_lines(&self) -> Result<Vec<String>> {
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct EntityResponse {
            id: String,
        }

        let response: Vec<EntityResponse> = self
            .http
            .get("https://api.tfl.gov.uk/Line/Mode/tube/Route")
            .await?
            .json()
            .await?;
        let lines = response.into_iter().map(|e| e.id).collect();
        Ok(lines)
    }

    pub async fn get_stations(&self, line: String) -> Result<Vec<TubeStation>> {
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct StopPointResponse {
            id: String,
            common_name: String,
            lat: f64,
            lon: f64,
            line_mode_groups: Vec<LineModeGroupResponse>,
            additional_properties: Vec<AdditionalPropertiesResponse>,
        }
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct LineModeGroupResponse {
            mode_name: String,
            line_identifier: Vec<String>,
        }

        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct AdditionalPropertiesResponse {
            category: String,
            key: String,
            value: String,
        }

        let response: Vec<StopPointResponse> = self
            .http
            .get(format!("https://api.tfl.gov.uk/Line/{}/StopPoints", line))
            .await?
            .json()
            .await?;
        let stations: Vec<TubeStation> = response
            .into_iter()
            .map(|s| {
                lazy_static! {
                    static ref RE: Regex = Regex::new(r".*,([A-Z0-9 ]+)").unwrap();
                }

                TubeStation {
                    id: s.id,
                    name: s.common_name,
                    postcode: s
                        .additional_properties
                        .into_iter()
                        .find(|p| p.category == "Address" && p.key == "Address")
                        .map(|p| p.value)
                        .and_then(|address| {
                            RE.captures(&address)
                                .and_then(|caps| caps.get(1))
                                .map(|m| m.as_str().to_owned())
                        }),
                    coordinates: (s.lon, s.lat),
                    lines: s
                        .line_mode_groups
                        .into_iter()
                        .find(|l| l.mode_name == "tube")
                        .unwrap()
                        .line_identifier
                        .into_iter()
                        .collect(),
                }
            })
            .collect();
        Ok(stations)
    }
}

#[cfg(test)]
mod tests {
    use super::TubeApi;
    use crate::lib::{tube::tube::TubeStation, util::globals::Globals};
    use std::collections::HashSet;

    #[tokio::test]
    async fn test_get_tube_lines() {
        let globals = Globals::new().await;
        let tube_api = TubeApi::new(&globals);
        let lines = tube_api.get_lines().await.unwrap();
        assert_eq!(
            lines,
            vec![
                "bakerloo",
                "central",
                "circle",
                "district",
                "hammersmith-city",
                "jubilee",
                "metropolitan",
                "northern",
                "piccadilly",
                "victoria",
                "waterloo-city"
            ]
        );
    }

    #[tokio::test]
    async fn test_get_stations() {
        let globals = Globals::new().await;
        let tube_api = TubeApi::new(&globals);
        let stations = tube_api
            .get_stations("waterloo-city".to_owned())
            .await
            .unwrap();
        assert_eq!(
            stations,
            vec![
                TubeStation {
                    id: "940GZZLUBNK".to_owned(),
                    name: "Bank Underground Station".to_owned(),
                    postcode: Some("EC3V 3LA".to_owned()),
                    coordinates: (-0.088899, 51.513356),
                    lines: HashSet::from([
                        "central".to_owned(),
                        "northern".to_owned(),
                        "waterloo-city".to_owned()
                    ]),
                },
                TubeStation {
                    id: "940GZZLUWLO".to_owned(),
                    name: "Waterloo Underground Station".to_owned(),
                    postcode: Some("SE1 7ND".to_owned()),
                    coordinates: (-0.11478, 51.503299),
                    lines: HashSet::from([
                        "bakerloo".to_owned(),
                        "jubilee".to_owned(),
                        "northern".to_owned(),
                        "waterloo-city".to_owned()
                    ])
                }
            ]
        );
    }
}
