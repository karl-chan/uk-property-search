use super::{db::Db, properties::Properties};
use log::Level;
use std::sync::Once;

static INIT: Once = Once::new();

pub struct Globals {
    pub db: Db,
    pub properties: Properties,
}

impl Globals {
    pub async fn new() -> Globals {
        INIT.call_once(|| {
            simple_logger::init_with_level(Level::Debug).unwrap();
        });
        let properties = Properties::new();
        let db = Db::new(&properties).await;
        Globals { db, properties }
    }
}
