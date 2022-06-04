use anyhow::{anyhow, Result};
use async_trait::async_trait;
use itertools::Itertools;
use reqwest::Response;
use rocket::serde::DeserializeOwned;

/// Decode response as json, but print the response body on failure.
#[async_trait]
pub trait DecodeJsonResponseExt {
    async fn json_or_err<T: DeserializeOwned>(self) -> Result<T>;
}

#[async_trait]
impl DecodeJsonResponseExt for Response {
    async fn json_or_err<T: DeserializeOwned>(self) -> Result<T> {
        let text = self.text().await?;
        serde_json::from_str(&text).map_err(|e| anyhow!("{}\nResponse body:\n{}", e, &text))
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
