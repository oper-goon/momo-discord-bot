pub mod easynews;

use anyhow::{Result, anyhow, ensure};
use serde_json::Value;
use std::time::Duration;

pub fn client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .build().expect("Static provider HTTP configuration")
}

/// Never return reqwest errors or remote error payloads: they can contain secrets.
pub async fn json(response: Result<reqwest::Response, reqwest::Error>) -> Result<Value> {
    let mut response = response.map_err(|_| anyhow!("Provider network request failed"))?;
    match response.status().as_u16() {
        401 | 403 => return Err(anyhow!("Provider authentication failed")),
        429 => return Err(anyhow!("Provider rate limited")),
        200..=299 => (),
        _ => return Err(anyhow!("Provider content unavailable")),
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| anyhow!("Provider response failed"))? {
        ensure!(bytes.len().saturating_add(chunk.len()) <= 2 * 1024 * 1024, "Provider response exceeded limit");
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| anyhow!("Invalid provider response"))
}

pub fn data(value: &Value) -> Result<&Value> {
    ensure!(value.get("success").and_then(Value::as_bool) == Some(true), "Provider request rejected");
    value.get("data").ok_or_else(|| anyhow!("Incomplete provider response"))
}
