#[path = "../lib/mod.rs"]
mod lib;

use lib::school::School;
use lib::util::{ext::MongoCollectionExt, globals::Globals};
use rocket::fs::FileServer;
use rocket::serde::json::Json;
use rocket::{Config, State};
use std::env;
use std::net::Ipv4Addr;

#[macro_use]
extern crate rocket;

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
        .mount("/api", routes![schools])
        .mount("/", FileServer::from("../uk-property-search-app/dist/spa"))
}
