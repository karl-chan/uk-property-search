use super::{db::Db, properties::Properties};

pub struct Globals {
    pub db: Db,
    pub properties: Properties,
}

impl Globals {
    pub async fn new() -> Globals {
        let properties = Properties::new();
        let db = Db::new(&properties).await;
        Globals { db, properties }
    }
}
