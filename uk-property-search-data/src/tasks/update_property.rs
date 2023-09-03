use crate::lib::{
    property::{
        aggregator::PropertyAggregator,
        estate_agents::rightmove::Rightmove,
        property::{PropertyAction, PropertySummary},
    },
    tube::TubeStation,
    util::ext::{MongoCollectionExt, VecResultExt},
    util::globals::Globals,
};
use anyhow::Result;
use chrono::Utc;
use futures::{
    future::{join, join_all},
    TryFutureExt,
};
use itertools::{iproduct, Itertools};
use log::info;
use mongodb::{bson::doc, options::FindOneAndUpdateOptions};

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
    let aggregator = PropertyAggregator {};

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

    async fn get_buy_and_rent_property_summary(
        rightmove: &Rightmove,
        aggregator: &PropertyAggregator,
        station_info: StationInfo,
        num_beds: u32,
        radius: f64,
    ) -> BuyAndRentPropertySummary {
        let (buy_properties_result, rent_properties_result) = join(
            rightmove.search(
                station_info.location_identifier.clone(),
                PropertyAction::Buy,
                num_beds,
                radius,
            ),
            rightmove.search(
                station_info.location_identifier.clone(),
                PropertyAction::Rent,
                num_beds,
                radius,
            ),
        )
        .await;
        let buy_properties = buy_properties_result.unwrap();
        let rent_properties = rent_properties_result.unwrap();

        let buy_and_rent_property_stats =
            aggregator.calculate_buy_and_rent_property_stats(buy_properties, rent_properties);

        info!("Got property stats for station: [{:?}] postcode: [{:?}]  num beds: [{:?}] radius: [{:?}]",
                station_info.station.name,
                station_info.station.postcode,
                 num_beds, radius
            );
        BuyAndRentPropertySummary {
            buy_summary: PropertySummary {
                postcode: station_info.station.postcode.clone(),
                coordinates: station_info.station.coordinates,
                action: PropertyAction::Buy as u8,
                num_beds,
                stats: buy_and_rent_property_stats.buy_stats,
            },
            rent_summary: PropertySummary {
                postcode: station_info.station.postcode.clone(),
                coordinates: station_info.station.coordinates,
                action: PropertyAction::Rent as u8,
                num_beds,
                stats: buy_and_rent_property_stats.rent_stats,
            },
        }
    }

    let all_buy_and_rent_property_summary = join_all(
        iproduct!(station_infos, 0..(MAX_BEDS + 1), [SEARCH_RADIUS]).map(
            |(station_info, num_beds, radius)| {
                get_buy_and_rent_property_summary(
                    &rightmove,
                    &aggregator,
                    station_info,
                    num_beds,
                    radius,
                )
            },
        ),
    )
    .await;
    let all_property_summary = all_buy_and_rent_property_summary
        .into_iter()
        .flat_map(|s| vec![s.buy_summary, s.rent_summary].into_iter())
        .collect_vec();

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
    globals
        .db
        .last_updated()
        .find_one_and_update_with_session(
            doc! {},
            doc! {"$set": {"property":  Utc::now().timestamp_millis() }},
            FindOneAndUpdateOptions::builder().upsert(true).build(),
            &mut session,
        )
        .await?;
    session.commit_transaction().await?;

    Ok(())
}

struct BuyAndRentPropertySummary {
    buy_summary: PropertySummary,
    rent_summary: PropertySummary,
}
