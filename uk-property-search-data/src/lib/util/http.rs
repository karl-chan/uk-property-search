use super::globals::Globals;
use reqwest::{
    header::{HeaderMap, ACCEPT_ENCODING, USER_AGENT},
    redirect::Policy,
    Client, IntoUrl,
};
use tokio::sync::Semaphore;

pub struct Http {
    client: Client,
    no_redirect_client: Client,
    semaphore: Semaphore,
}

pub struct HttpOptions {
    max_parallel_connections: Option<usize>,
}

impl Http {
    pub fn new(globals: &Globals, options: Option<HttpOptions>) -> Http {
        let default_max_parallel_connections = globals
            .properties
            .get_int("http.max.parallel.connections")
            .try_into()
            .unwrap();

        let default_headers = {
            let mut headers = HeaderMap::new();
            headers.insert(USER_AGENT, "Mozilla/5.0 (Windows NT 6.2; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/56.0.2924.87 Safari/537.36".parse().unwrap());
            headers.insert(ACCEPT_ENCODING, "gzip".parse().unwrap());
            headers
        };

        Http {
            client: reqwest::Client::builder()
                .default_headers(default_headers.clone())
                .build()
                .unwrap(),
            no_redirect_client: reqwest::Client::builder()
                .default_headers(default_headers.clone())
                .redirect(Policy::none())
                .build()
                .unwrap(),
            semaphore: Semaphore::new(
                options
                    .and_then(|o| o.max_parallel_connections)
                    .unwrap_or(default_max_parallel_connections),
            ),
        }
    }

    pub async fn get<U: IntoUrl>(&self, url: U) -> Result<reqwest::Response, reqwest::Error> {
        self.get_with_options(url, &[], true).await
    }

    pub async fn get_with_options<U: IntoUrl>(
        &self,
        url: U,
        query: &[(&str, &str)],
        follow_redirects: bool,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let permit = self.semaphore.acquire();
        let client = if follow_redirects {
            &self.client
        } else {
            &self.no_redirect_client
        };
        let response = client.get(url).query(query).send().await;
        drop(permit);
        response
    }
}

#[cfg(test)]
mod tests {
    use futures::future::join_all;
    use reqwest::StatusCode;

    use crate::lib::util::globals::Globals;

    use super::{Http, HttpOptions};

    #[tokio::test]
    async fn test_get() {
        let globals = Globals::new().await;
        let http = Http::new(&globals, None);
        let response = http.get("https://www.duckduckgo.com").await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_parallel_get() {
        let globals = Globals::new().await;
        let http = Http::new(
            &globals,
            Some(HttpOptions {
                max_parallel_connections: Some(5),
            }),
        );
        let futures = (0..20)
            .into_iter()
            .map(|_| http.get("https://www.duckduckgo.com"))
            .collect::<Vec<_>>();
        let tasks = join_all(futures).await;

        assert_eq!(tasks.len(), 20);
        for task in tasks {
            let response = task.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
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
