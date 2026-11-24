pub mod easynews;
pub mod torbox;

use anyhow::{Result, anyhow, ensure};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug)]
pub struct RateLimited(pub u64);
impl std::fmt::Display for RateLimited {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Provider rate limited")
    }
}
impl std::error::Error for RateLimited {}

pub fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Static provider HTTP configuration")
}

/// Never return reqwest errors or remote error payloads: they can contain secrets.
pub async fn json(response: Result<reqwest::Response, reqwest::Error>) -> Result<Value> {
    let mut response = response.map_err(|_| anyhow!("Provider network request failed"))?;
    match response.status().as_u16() {
        401 | 403 => return Err(anyhow!("Provider authentication failed")),
        429 => {
            let seconds = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(120);
            return Err(RateLimited(seconds.min(86400)).into());
        }
        200..=299 => (),
        _ => return Err(anyhow!("Provider content unavailable")),
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| anyhow!("Provider response failed"))?
    {
        ensure!(
            bytes.len().saturating_add(chunk.len()) <= 2 * 1024 * 1024,
            "Provider response exceeded limit"
        );
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| anyhow!("Invalid provider response"))
}

pub fn data(value: &Value) -> Result<&Value> {
    ensure!(
        value.get("success").and_then(Value::as_bool) == Some(true),
        "Provider request rejected"
    );
    value
        .get("data")
        .ok_or_else(|| anyhow!("Incomplete provider response"))
}

#[cfg(test)]
pub async fn mock_api(
    responses: Vec<(&'static str, String)>,
) -> (String, tokio::task::JoinHandle<Vec<String>>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        let mut requests = vec![];
        for (status, body) in responses {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = vec![];
            loop {
                let byte = socket.read_u8().await.unwrap();
                request.push(byte);
                assert!(request.len() < 16384);
                if request.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            requests.push(String::from_utf8(request).unwrap());
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(response.as_bytes()).await.unwrap();
        }
        requests
    });
    (address, server)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn auth_rate_limit_expired_links_and_invalid_payloads_are_safe() {
        for (status, expected) in [
            ("401 Unauthorized", "authentication"),
            ("429 Too Many Requests", "rate limited"),
            ("410 Gone", "unavailable"),
            ("200 OK", "Invalid provider"),
        ] {
            let (base, server) = mock_api(vec![(status, "private-token-not-json".into())]).await;
            let error = json(client().get(base).send().await)
                .await
                .unwrap_err()
                .to_string();
            assert!(error.contains(expected), "{error}");
            assert!(!error.contains("private-token"));
            server.await.unwrap();
        }
    }
}
