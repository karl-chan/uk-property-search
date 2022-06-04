use crate::lib::{
    property::{
        estate_agents::rightmove::Rightmove,
        property::{PropertyAction, PropertyStatsProvider, PropertySummary},
    },
    tube::TubeStation,
    util::globals::Globals,
    util::lang::VecResultExt,
};
use anyhow::Result;
use futures::{future::join_all, StreamExt, TryFutureExt};
use itertools::{iproduct, Itertools};
use log::debug;
use mongodb::bson::doc;

pub async fn update_property(globals: &Globals) -> Result<()> {
    let results: Vec<core::result::Result<TubeStation, _>> =
        globals.db.tube().find(None, None).await?.collect().await;
    let tube_stations: Vec<TubeStation> = results.into_iter().map(|r| r.unwrap()).collect();
    let station_postcodes: Vec<String> = tube_stations
        .iter()
        .map(|station| station.postcode.to_owned())
        .collect_vec();
    let station_coordinates: Vec<(f64, f64)> = tube_stations
        .iter()
        .map(|station| station.coordinates)
        .collect_vec();

    let rightmove = Rightmove::new(&globals);
    let all_property_summary = join_all(
        iproduct!(
            station_postcodes,
            station_coordinates,
            [PropertyAction::Buy, PropertyAction::Rent],
            0..4,
            [0.25]
        )
        .map(|(postcode, coordinates, action, num_beds, radius)| {
            // debug!("Querying rightmove for station: [{:?}] action: [{:?}] num beds: [{:?}] radius: [{:?}]", &station_name, action, num_beds, radius);
            rightmove
                .get_stats(postcode.clone(), action, num_beds, radius)
                .map_ok(move |stats|  {
                    debug!("Got stats for station: [{:?}] action: [{:?}] num beds: [{:?}] radius: [{:?}]", &postcode, action, num_beds, radius);
                    PropertySummary {
                        postcode,
                        coordinates,
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
