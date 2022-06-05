#[path = "../lib/mod.rs"]
mod lib;

use lib::property::property::PropertySummary;
use lib::school::School;
use lib::tube::TubeStation;
use lib::util::{ext::MongoCollectionExt, globals::Globals};
use rocket::fs::FileServer;
use rocket::serde::json::Json;
use rocket::{Config, State};
use std::env;
use std::net::Ipv4Addr;

#[macro_use]
extern crate rocket;

#[get("/property")]
async fn property(state: &State<Globals>) -> Json<Vec<PropertySummary>> {
    let property = state.inner().db.property().find_to_vec().await;
    Json(property)
}

#[get("/tube-stations")]
async fn tube_stations(state: &State<Globals>) -> Json<Vec<TubeStation>> {
    let tube_stations = state.inner().db.tube().find_to_vec().await;
    Json(tube_stations)
}

#[get("/schools")]
async fn schools(state: &State<Globals>) -> Json<Vec<School>> {
    let schools = state.inner().db.schools().find_to_vec().await;
    Json(schools)
}

#[launch]
async fn rocket() -> _ {
    let globals = Globals::new().await;
    let config = Config {
        address: Ipv4Addr::new(0, 0, 0, 0).into(),
        port: env::var("PORT").map_or(
            globals
                .properties
                .get_int("server.default.port")
                .try_into()
                .unwrap(),
            |s| s.parse().unwrap(),
        ),
        ..Config::default()
    };

    rocket::custom(&config)
        .manage(globals)
        .mount("/api", routes![property, tube_stations, schools])
        .mount("/", FileServer::from("../uk-property-search-app/dist/spa"))
}
