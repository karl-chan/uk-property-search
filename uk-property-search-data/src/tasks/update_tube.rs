use crate::lib::{tube::TubeStation, util::globals::Globals};
use anyhow::Result;
use itertools::{multizip, Itertools};
use mongodb::bson::doc;
use polars::{io::SerReader, prelude::CsvReader};

pub async fn update_tube(globals: &Globals) -> Result<()> {
    let stations_df = CsvReader::from_path("assets/London stations.csv")?.finish()?;

    let stations = stations_df.column("Station")?.utf8()?;
    let latitudes = stations_df.column("Latitude")?.f64()?;
    let longitudes = stations_df.column("Longitude")?.f64()?;
    let zones = stations_df.column("Zone")?.utf8()?;
    let postcodes = stations_df.column("Postcode")?.utf8()?;

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
    session.commit_transaction().await?;

    Ok(())
}
