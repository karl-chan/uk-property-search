use std::collections::HashSet;

use crate::lib::{tube::TubeStation, util::globals::Globals};
use anyhow::Result;
use chrono::Utc;
use itertools::{multizip, Itertools};
use mongodb::{bson::doc, options::FindOneAndUpdateOptions};
use polars::{io::SerReader, prelude::CsvReader};

pub async fn update_tube(globals: &Globals) -> Result<()> {
    let stations_df = CsvReader::from_path("assets/London stations.csv")?.finish()?;
    let lines_df = CsvReader::from_path("assets/London tube lines.csv")?.finish()?;

    let stations = stations_df.column("Station")?.utf8()?;
    let latitudes = stations_df.column("Latitude")?.f64()?;
    let longitudes = stations_df.column("Longitude")?.f64()?;
    let zones = stations_df.column("Zone")?.utf8()?;
    let postcodes = stations_df.column("Postcode")?.utf8()?;

    let lines = lines_df.column("Tube Line")?.utf8()?;
    let from_stations = lines_df.column("From Station")?.utf8()?;
    let to_stations = lines_df.column("To Station")?.utf8()?;

    let station_lines_lookup = multizip((lines, from_stations, to_stations))
        .flat_map(|(line, from_station, to_station)| {
            [
                (from_station.unwrap().to_owned(), line.unwrap().to_owned()),
                (to_station.unwrap().to_owned(), line.unwrap().to_owned()),
            ]
            .into_iter()
        })
        .into_grouping_map()
        .collect::<HashSet<_>>();

    let tube_stations: Vec<TubeStation> =
        multizip((stations, latitudes, longitudes, zones, postcodes))
            .map(
                |(station, latitude, longitude, zone, postcode)| TubeStation {
                    name: station.unwrap().to_owned(),
                    zone: zone
                        .unwrap()
                        .split(",")
                        .filter_map(|z| z.parse::<u8>().ok())
                        .collect_vec(),
                    postcode: postcode.unwrap().to_owned(),
                    coordinates: (longitude.unwrap(), latitude.unwrap()),
                    lines: station_lines_lookup
                        .get(station.unwrap())
                        .expect(&format!(
                            "Station [{}] missing from station_lines_lookup!",
                            station.unwrap()
                        ))
                        .to_owned(),
                },
            )
            .collect();

    let mut session = globals.db.client.start_session(None).await?;
    session.start_transaction(None).await?;

    globals
        .db
        .tube()
        .delete_many_with_session(doc! {}, None, &mut session)
        .await?;
    globals
        .db
        .tube()
        .insert_many_with_session(tube_stations, None, &mut session)
        .await?;
    globals
        .db
        .last_updated()
        .find_one_and_update_with_session(
            doc! {},
            doc! {"$set": {"tube":  Utc::now().timestamp_millis() }},
            FindOneAndUpdateOptions::builder().upsert(true).build(),
            &mut session,
        )
        .await?;
    session.commit_transaction().await?;

    Ok(())
}
