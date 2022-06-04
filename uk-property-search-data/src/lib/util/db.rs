use super::properties::Properties;
use crate::lib::{property::property::PropertySummary, school::School, tube::TubeStation};
use mongodb::{options::ClientOptions, Client, Collection, Database};

pub struct Db {
    pub client: Client,
    pub database: Database,
}

impl Db {
    pub async fn new(properties: &Properties) -> Db {
        let client = Client::with_options(
            ClientOptions::parse(properties.get_string("db.mongo.uri"))
                .await
                .unwrap(),
        )
        .unwrap();
        let database = client.database(&properties.get_string("db.mongo.name"));
        Db { client, database }
    }

    pub fn property(&self) -> Collection<PropertySummary> {
        self.database.collection("property")
    }

    pub fn schools(&self) -> Collection<School> {
        self.database.collection("schools")
    }

    pub fn tube(&self) -> Collection<TubeStation> {
        self.database.collection("tube")
    }
}

#[cfg(test)]
mod tests {
    use crate::lib::util::properties::Properties;

    use super::Db;

    #[tokio::test]
    async fn test_connection() {
        let properties = Properties::new();
        let db = Db::new(&properties).await;
        assert_eq!(db.database.name(), "uk-property-search");
    }
}
