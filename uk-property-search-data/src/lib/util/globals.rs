use super::{db::Db, properties::Properties};
use log::Level;
use std::{str::FromStr, sync::Once};

static INIT: Once = Once::new();

pub struct Globals {
    pub db: Db,
    pub properties: Properties,
}

impl Globals {
    pub async fn new() -> Globals {
        let properties = Properties::new();
        let db = Db::new(&properties).await;

        INIT.call_once(|| set_logger(&properties));

        Globals { db, properties }
    }
}

fn set_logger(properties: &Properties) {
    let level_str = properties.get_string("log.level");
    let level = Level::from_str(&level_str)
        .expect(format!("[{:?}] is not a valid log level!", &level_str).as_str());
    simple_logger::init_with_level(level).unwrap();
}
