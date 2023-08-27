use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::StreamExt;
use itertools::Itertools;
use mongodb::Collection;
use reqwest::Response;
use rocket::serde::DeserializeOwned;

/// Decode response as json, but print the response body on failure.
#[async_trait]
pub trait DecodeJsonResponseExt {
    async fn json_or_err<T: DeserializeOwned>(self, context: &str) -> Result<T>;
}

#[async_trait]
impl DecodeJsonResponseExt for Response {
    async fn json_or_err<T: DeserializeOwned>(self, context: &str) -> Result<T> {
        let url = self.url().clone();
        let status_code = self.status().as_u16();
        let text = self.text().await?;
        serde_json::from_str(&text).map_err(|e| {
            anyhow!(
                "{}\nURL: {}\nStatus code: {}\nResponse body:\n{}\nContext: {}",
                e,
                url,
                status_code,
                &text,
                context,
            )
        })
    }
}

/// Unwrap Vec<Result<T>> into Vec<T>, panicking on failure.
pub trait VecResultExt<T> {
    fn unwrap_all(self) -> Vec<T>;
}

impl<T> VecResultExt<T> for Vec<Result<T>> {
    fn unwrap_all(self) -> Vec<T> {
        self.into_iter().map(|result| result.unwrap()).collect_vec()
    }
}

#[async_trait]
pub trait MongoCollectionExt<T> {
    async fn find_to_vec(&self) -> Vec<T>;
}

#[async_trait]
impl<T> MongoCollectionExt<T> for Collection<T>
where
    T: DeserializeOwned + Unpin + Send + Sync,
{
    async fn find_to_vec(&self) -> Vec<T> {
        self.find(None, None)
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect()
    }
}
