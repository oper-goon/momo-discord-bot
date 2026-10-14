use std::time::Duration;

const RELEASE_URL: &str = "https://api.github.com/repos/oper-goon/momo-discord-bot/releases/latest";
const MAX_RESPONSE_BYTES: usize = 64 * 1024;

pub async fn status() -> String {
    tokio::time::timeout(Duration::from_secs(3), fetch())
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "Unavailable".to_owned())
}

async fn fetch() -> Option<String> {
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(3))
        .user_agent(concat!("Momo/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?;
    let mut response = client
        .get(RELEASE_URL)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .ok()?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Some("Unavailable (no public release)".to_owned());
    }
    if !response.status().is_success()
        || response
            .content_length()
            .is_some_and(|n| n > MAX_RESPONSE_BYTES as u64)
    {
        return None;
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.ok()? {
        if chunk.len() > MAX_RESPONSE_BYTES - bytes.len() {
            return None;
        }
        bytes.extend_from_slice(&chunk);
    }
    release_status(&bytes, env!("CARGO_PKG_VERSION"))
}

// Stable release tags must be canonical MAJOR.MINOR.PATCH, optionally prefixed
// with v. Reject other tags instead of displaying untrusted terminal content.
fn stable_version(tag: &str) -> Option<[u64; 3]> {
    let tag = tag.strip_prefix('v').unwrap_or(tag);
    if tag.len() > 24 {
        return None;
    }
    let parts: Vec<_> = tag.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let mut version = [0; 3];
    for (index, part) in parts.iter().enumerate() {
        if part.is_empty()
            || !part.bytes().all(|b| b.is_ascii_digit())
            || (part.len() > 1 && part.starts_with('0'))
        {
            return None;
        }
        version[index] = part.parse().ok()?;
    }
    Some(version)
}

fn release_status(bytes: &[u8], current: &str) -> Option<String> {
    let release: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    if release.get("draft")?.as_bool()? || release.get("prerelease")?.as_bool()? {
        return None;
    }
    let latest = stable_version(release.get("tag_name")?.as_str()?)?;
    let current = stable_version(current)?;
    Some(if latest > current {
        format!("Available: v{}.{}.{}", latest[0], latest[1], latest[2])
    } else if latest == current {
        "Up to date".to_owned()
    } else {
        "Ahead of latest release".to_owned()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions_numerically() {
        let status = |tag: &str| {
            let body = serde_json::json!({"tag_name": tag, "draft": false, "prerelease": false});
            release_status(body.to_string().as_bytes(), "0.2.0")
        };
        assert_eq!(status("v0.10.0").as_deref(), Some("Available: v0.10.0"));
        assert_eq!(status("0.2.0").as_deref(), Some("Up to date"));
        assert_eq!(status("v0.1.0").as_deref(), Some("Ahead of latest release"));
    }

    #[test]
    fn rejects_untrusted_or_unstable_release_data() {
        for tag in ["v0.3.0-rc.1", "v0.03.0", "0.3", "\x1b[31m0.3.0", "latest"] {
            assert!(stable_version(tag).is_none());
        }
        for body in [
            br#"{"tag_name":"v1.0.0","draft":true,"prerelease":false}"#.as_slice(),
            br#"{"tag_name":"v1.0.0","draft":false,"prerelease":true}"#.as_slice(),
            br#"{"tag_name":"v1.0.0"}"#.as_slice(),
            b"invalid",
        ] {
            assert!(release_status(body, "0.2.0").is_none());
        }
    }
}
