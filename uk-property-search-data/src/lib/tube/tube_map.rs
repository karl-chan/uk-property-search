use super::{tube::TubeStation, tube_api::TubeApi};
use anyhow::Result;
use futures::future::join_all;
use std::collections::HashSet;

pub struct TubeMap<'a> {
    tube_api: &'a TubeApi,
}

impl<'a> TubeMap<'a> {
    pub fn new(tube_api: &'a TubeApi) -> TubeMap<'a> {
        TubeMap { tube_api }
    }
    pub async fn get_tube_stations(&self) -> Result<HashSet<TubeStation>> {
        let lines = self.tube_api.get_lines().await?;
        let line_stations = join_all(
            lines
                .into_iter()
                .map(|line| self.tube_api.get_stations(line)),
        )
        .await;
        let all_stations = line_stations
            .into_iter()
            .flat_map(|stations_result| stations_result.unwrap())
            .collect();
        Ok(all_stations)
    }
}

#[cfg(test)]
mod tests {
    use super::TubeMap;
    use crate::lib::{tube::tube_api::TubeApi, util::globals::Globals};
    use more_asserts::assert_gt;

    #[tokio::test]
    async fn test_get_tube_stations() {
        let globals = Globals::new().await;
        let tube_api = TubeApi::new(&globals);
        let tube_map = TubeMap::new(&tube_api);
        let tube_stations = tube_map.get_tube_stations().await.unwrap();
        assert_gt!(tube_stations.len(), 270);
    }
}
