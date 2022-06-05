use crate::lib::{
    property::{
        estate_agents::rightmove::Rightmove,
        property::{PropertyAction, PropertySummary},
    },
    tube::TubeStation,
    util::ext::{MongoCollectionExt, VecResultExt},
    util::globals::Globals,
};
use anyhow::Result;
use futures::{future::join_all, TryFutureExt};
use itertools::iproduct;
use log::info;
use mongodb::bson::doc;

// Only consider Studio - 3 bedroom flats
const MAX_BEDS: u32 = 3;

// Only consider 0.25 miles radius from train stations
const SEARCH_RADIUS: f64 = 0.25;

pub async fn update_property(globals: &Globals) -> Result<()> {
    #[derive(Clone)]
    struct StationInfo {
        station: TubeStation,
        location_identifier: String,
    }

    let rightmove = Rightmove::new(&globals);

    let tube_stations: Vec<TubeStation> = globals.db.tube().find_to_vec().await;
    let station_infos: Vec<StationInfo> = join_all(tube_stations.into_iter().map(|station| {
        rightmove
            .get_location_identifier(station.postcode.to_owned())
            .map_ok(|location_identifier| StationInfo {
                station,
                location_identifier,
            })
    }))
    .await
    .unwrap_all();

    let all_property_summary = join_all(
        iproduct!(
            [&rightmove],
            station_infos,
            [PropertyAction::Buy, PropertyAction::Rent],
            0..(MAX_BEDS+1),
            [SEARCH_RADIUS]
        )
        .map(|(rightmove, station_info, action, num_beds, radius)| {
            rightmove
                .search(station_info.location_identifier, action, num_beds, radius)
                .map_ok(move |properties| {
                    let stats = rightmove.to_stats(properties);
                    info!("Got property stats for station: [{:?}], postcode: [{:?}] action: [{:?}] num beds: [{:?}] radius: [{:?}]",
                        station_info.station.name,
                        station_info.station.postcode,
                        action, num_beds, radius
                    );
                    PropertySummary {
                        postcode: station_info.station.postcode,
                        coordinates: station_info.station.coordinates,
                        stats,
                    }
                })
        }),
    )
    .await
    .unwrap_all();

    let mut session = globals.db.client.start_session(None).await?;
    session.start_transaction(None).await?;

    globals
        .db
        .property()
        .delete_many_with_session(doc! {}, None, &mut session)
        .await?;
    globals
        .db
        .property()
        .insert_many_with_session(all_property_summary, None, &mut session)
        .await?;
    session.commit_transaction().await?;

    Ok(())
}
