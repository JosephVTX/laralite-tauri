use crate::error::{Ctx, Res};
use reqwest::{Client, RequestBuilder};
use std::sync::OnceLock;
use std::time::Duration;

pub fn client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .user_agent("Mozilla/5.0 Laralite/0.1")
            .connect_timeout(Duration::from_secs(20))
            .build()
            .expect("cliente http")
    })
}

/// GET con Referer opcional (Apache Lounge lo exige).
pub fn get(url: &str, referer: Option<&str>) -> RequestBuilder {
    let req = client().get(url);
    match referer {
        Some(r) => req.header("Referer", r),
        None => req,
    }
}

pub async fn text(url: &str, referer: Option<&str>) -> Res<String> {
    get(url, referer).send().await.ctx(url)?.error_for_status().ctx(url)?.text().await.ctx(url)
}

pub async fn json(url: &str) -> Res<serde_json::Value> {
    get(url, None).send().await.ctx(url)?.error_for_status().ctx(url)?.json().await.ctx(url)
}
