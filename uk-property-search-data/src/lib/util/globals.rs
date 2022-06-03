use std::sync::Once;

use super::{db::Db, properties::Properties};
use log::Level;

static INIT: Once = Once::new();

pub struct Globals {
    pub db: Db,
    pub properties: Properties,
}

impl Globals {
    pub async fn new() -> Globals {
        INIT.call_once(|| {
            simple_logger::init_with_level(Level::Info).unwrap();
        });
        let properties = Properties::new();
        let db = Db::new(&properties).await;
        Globals { db, properties }
    }
}
