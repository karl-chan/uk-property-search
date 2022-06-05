use super::globals::Globals;
use log::debug;
use reqwest::{
    header::{HeaderMap, USER_AGENT},
    redirect::Policy,
    IntoUrl, Response,
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Error};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::fmt::Debug;
use tokio::sync::Semaphore;

pub struct Http {
    client: ClientWithMiddleware,
    no_redirect_client: ClientWithMiddleware,
    semaphore: Semaphore,
}

pub struct HttpOptions {
    max_parallel_connections: Option<usize>,
    max_retry_count: Option<u32>,
}

impl Http {
    pub fn new(globals: &Globals, options: Option<HttpOptions>) -> Http {
        let default_max_parallel_connections = globals
            .properties
            .get_int("http.max.parallel.connections")
            .try_into()
            .unwrap();

        let default_max_retry_count = globals
            .properties
            .get_int("http.max.retry.count")
            .try_into()
            .unwrap();

        let default_headers = {
            let mut headers = HeaderMap::new();
            headers.insert(USER_AGENT, "Mozilla/5.0 (Windows NT 6.2; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/56.0.2924.87 Safari/537.36".parse().unwrap());
            headers
        };

        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(
            options
                .as_ref()
                .and_then(|o| o.max_retry_count)
                .unwrap_or(default_max_retry_count),
        );

        Http {
            client: ClientBuilder::new(
                reqwest::Client::builder()
                    .default_headers(default_headers.clone())
                    .build()
                    .unwrap(),
            )
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build(),
            no_redirect_client: ClientBuilder::new(
                reqwest::Client::builder()
                    .default_headers(default_headers.clone())
                    .redirect(Policy::none())
                    .build()
                    .unwrap(),
            )
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build(),
            semaphore: Semaphore::new(
                options
                    .as_ref()
                    .and_then(|o| o.max_parallel_connections)
                    .unwrap_or(default_max_parallel_connections),
            ),
        }
    }

    pub async fn get<U: IntoUrl + Debug>(&self, url: U) -> Result<Response, Error> {
        self.get_with_options(url, &[], true).await
    }

    pub async fn get_with_options<U: IntoUrl + Debug>(
        &self,
        url: U,
        query: &[(&str, &str)],
        follow_redirects: bool,
    ) -> Result<Response, Error> {
        let permit = self.semaphore.acquire().await.unwrap();
        let client = if follow_redirects {
            &self.client
        } else {
            &self.no_redirect_client
        };
        debug!(
            "Sending GET request to url: [{:?}] with query: [{:?}] and follow redirects: [{:?}]",
            &url, query, follow_redirects
        );
        let response = client.get(url).query(query).send().await;
        drop(permit);
        response
    }
}

#[cfg(test)]
mod tests {
    use super::{Http, HttpOptions};
    use crate::lib::util::globals::Globals;
    use futures::future::join_all;
    use itertools::Itertools;
    use reqwest::StatusCode;

    #[tokio::test]
    async fn test_get() {
        let globals = Globals::new().await;
        let http = Http::new(&globals, None);
        let response = http.get("https://www.duckduckgo.com").await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.text().await.unwrap().contains("<!DOCTYPE html>"))
    }

    #[tokio::test]
    async fn test_parallel_get() {
        let globals = Globals::new().await;
        let http = Http::new(
            &globals,
            Some(HttpOptions {
                max_parallel_connections: Some(5),
                max_retry_count: None,
            }),
        );
        let futures = (0..20)
            .into_iter()
            .map(|_| http.get("https://www.duckduckgo.com"))
            .collect_vec();
        let tasks = join_all(futures).await;

        assert_eq!(tasks.len(), 20);
        for task in tasks {
            let response = task.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            assert!(response.text().await.unwrap().contains("<!DOCTYPE html>"))
        }
    }

    #[tokio::test]
    async fn test_get_301() {
        let globals = Globals::new().await;
        let http = Http::new(&globals, None);
        let response = http
            .get_with_options("https://bbc.co.uk/301", &[], false)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::MOVED_PERMANENTLY);
    }

    #[tokio::test]
    async fn test_get_404() {
        let globals = Globals::new().await;
        let http = Http::new(&globals, None);
        let response = http.get("https://bbc.co.uk/404").await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
