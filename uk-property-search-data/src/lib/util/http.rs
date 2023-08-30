use super::globals::Globals;
use log::debug;
use reqwest::{
    header::{HeaderMap, HeaderValue, REFERER, USER_AGENT},
    redirect::Policy,
    IntoUrl, Method, Response,
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Error};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::Serialize;
use serde_json::Value;
use std::fmt::Debug;
use tokio::sync::Semaphore;

pub struct Http {
    client: ClientWithMiddleware,
    no_redirect_client: ClientWithMiddleware,
    semaphore: Semaphore,
}

pub struct HttpOptions {
    pub max_parallel_connections: Option<usize>,
    pub max_retry_count: Option<u32>,
    pub referer: Option<String>,
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
            options
                .as_ref()
                .and_then(|o| o.referer.as_ref())
                .map(|r| headers.insert(REFERER, HeaderValue::from_str(r).unwrap()));
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
        self.request(
            url,
            Method::GET,
            None,
            None::<&[(&str, &str)]>,
            None::<&Value>,
            true,
        )
        .await
    }

    pub async fn get_with_options<U: IntoUrl + Debug>(
        &self,
        url: U,
        query: &[(&str, &str)],
        follow_redirects: bool,
    ) -> Result<Response, Error> {
        self.request(
            url,
            Method::GET,
            Some(query),
            None::<&[(&str, &str)]>,
            None::<&Value>,
            follow_redirects,
        )
        .await
    }

    pub async fn post_with_form<U: IntoUrl + Debug, F: Serialize + ?Sized + Debug>(
        &self,
        url: U,
        form: &F,
    ) -> Result<Response, Error> {
        self.request(url, Method::POST, None, Some(form), None::<&Value>, true)
            .await
    }

    pub async fn post_with_json<U: IntoUrl + Debug, J: Serialize + ?Sized + Debug>(
        &self,
        url: U,
        json: &J,
    ) -> Result<Response, Error> {
        self.request(
            url,
            Method::POST,
            None,
            None::<&[(&str, &str)]>,
            Some(json),
            true,
        )
        .await
    }

    async fn request<
        U: IntoUrl + Debug,
        F: Serialize + ?Sized + Debug,
        J: Serialize + ?Sized + Debug,
    >(
        &self,
        url: U,
        method: Method,
        query: Option<&[(&str, &str)]>,
        form: Option<&F>,
        json: Option<&J>,
        follow_redirects: bool,
    ) -> Result<Response, Error> {
        let permit = self.semaphore.acquire().await.unwrap();
        let log_request_prefix =
            self.prepare_log_request(&url, &method, &query, &form, &json, follow_redirects);
        let client = if follow_redirects {
            &self.client
        } else {
            &self.no_redirect_client
        };
        let mut request = client.request(method, url);
        if let Some(q) = query {
            request = request.query(q);
        }
        if let Some(f) = form {
            request = request.form(f);
        }
        if let Some(j) = json {
            request = request.json(j);
        }
        let response = request.send().await;
        self.log_request(&log_request_prefix, &response.as_ref().unwrap());
        drop(permit);
        response
    }

    fn prepare_log_request<
        U: IntoUrl + Debug,
        F: Serialize + ?Sized + Debug,
        J: Serialize + ?Sized + Debug,
    >(
        &self,
        url: &U,
        method: &Method,
        query: &Option<&[(&str, &str)]>,
        form: &Option<&F>,
        json: &Option<&J>,
        follow_redirects: bool,
    ) -> String {
        let optional_params = {
            let mut parts = vec![];
            if let Some(q) = query {
                parts.push(format!("with query: [{:?}], ", q));
            }
            if let Some(f) = form {
                parts.push(format!("with form: [{:?}], ", f));
            }
            if let Some(j) = json {
                parts.push(format!("with json: [{:?}], ", j));
            }
            parts.concat()
        };
        format!(
            "[{:?}] request to url: [{:?}] {}and follow redirects: [{:?}]",
            method, url, optional_params, follow_redirects
        )
    }

    fn log_request(&self, log_statement_prefix: &str, response: &Response) {
        debug!(
            "{} returned status code [{:?}].",
            log_statement_prefix,
            response.status()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{Http, HttpOptions};
    use crate::lib::util::globals::Globals;
    use futures::future::join_all;
    use itertools::Itertools;
    use reqwest::StatusCode;
    use serde_json::Value;

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
                referer: None,
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

    #[tokio::test]
    async fn test_post_with_form() {
        let globals = Globals::new().await;
        let http = Http::new(&globals, None);
        let response = http
            .post_with_form("https://httpbin.org/post", &[("myKey", "myValue")])
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.json::<Value>().await.unwrap()["form"]["myKey"],
            "myValue"
        );
    }

    #[tokio::test]
    async fn test_post_with_json() {
        let globals = Globals::new().await;
        let http = Http::new(&globals, None);
        let response = http
            .post_with_json("https://httpbin.org/post", "myString")
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.json::<Value>().await.unwrap()["json"], "myString");
    }
}
