#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct ModuleDescriptor {
    pub id: &'static str,
    pub display_name: &'static str,
    pub version: &'static str,
    pub required: bool,
    pub default_enabled: bool,
    pub required_services: Vec<&'static str>,
    pub provided_services: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    Allow,
    Deny,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PermissionRule {
    pub id: i64,
    pub target: String,
    pub subject_kind: String,
    pub subject_id: Option<String>,
    pub channel_id: Option<String>,
    pub effect: Effect,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackRef {
    pub provider: String,
    pub canonical_url: String,
    pub title: String,
    pub duration_ms: Option<u64>,
    pub requested_by: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MusicSettings {
    pub dj_role_id: Option<String>,
    pub default_volume_percent: u16,
    pub max_queue_length: usize,
    pub idle_disconnect_seconds: u64,
    pub empty_channel_disconnect_seconds: u64,
    pub announce_now_playing: bool,
}

impl Default for MusicSettings {
    fn default() -> Self {
        Self {
            dj_role_id: None,
            default_volume_percent: 100,
            max_queue_length: 500,
            idle_disconnect_seconds: 300,
            empty_channel_disconnect_seconds: 120,
            announce_now_playing: true,
        }
    }
}
impl MusicSettings {
    pub fn validate(&self, limit: usize) -> anyhow::Result<()> {
        anyhow::ensure!(self.default_volume_percent <= 200, "Volume must be 0–200");
        anyhow::ensure!(
            (1..=limit).contains(&self.max_queue_length),
            "Queue limit exceeds process limits"
        );
        anyhow::ensure!(
            (10..=86400).contains(&self.idle_disconnect_seconds),
            "Idle timeout must be 10–86400 seconds"
        );
        anyhow::ensure!(
            (10..=86400).contains(&self.empty_channel_disconnect_seconds),
            "Empty-channel timeout must be 10–86400 seconds"
        );
        if let Some(id) = &self.dj_role_id {
            anyhow::ensure!(
                id.parse::<u64>()
                    .is_ok_and(|v| v > 0 && v.to_string() == *id),
                "Invalid DJ role ID"
            );
        }
        Ok(())
    }
}
