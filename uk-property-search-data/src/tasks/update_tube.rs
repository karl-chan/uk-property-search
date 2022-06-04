use crate::lib::{
    tube::{tube_api::TubeApi, tube_map::TubeMap},
    util::globals::Globals,
};
use anyhow::Result;
use mongodb::bson::doc;

pub async fn update_tube(globals: &Globals) -> Result<()> {
    let tube_api = TubeApi::new(globals);
    let tube_map = TubeMap::new(&tube_api);
    let tube_stations = tube_map.get_tube_stations().await?;

    let mut session = globals.db.client.start_session(None).await?;
    session.start_transaction(None).await?;

    globals
        .db
        .tube()
        .delete_many_with_session(doc! {}, None, &mut session)
        .await?;
    globals
        .db
        .tube()
        .insert_many_with_session(tube_stations, None, &mut session)
        .await?;
    session.commit_transaction().await?;

    Ok(())
}
