#![forbid(unsafe_code)]

mod commands;
mod service;

pub use service::DownloadService;

use anyhow::Result;
use async_trait::async_trait;
use bot_core::{BotData, Lifecycle, RegisteredModule};
use std::sync::Arc;

struct DownloadLifecycle {
    service: Arc<DownloadService>,
}

#[async_trait]
impl Lifecycle for DownloadLifecycle {
    async fn health(&self) -> Result<String> {
        Ok(if self.service.health().await {
            "healthy".into()
        } else {
            "degraded: yt-dlp unavailable".into()
        })
    }

    async fn shutdown(&self, _: &BotData) -> Result<()> {
        self.service.cancel();
        Ok(())
    }
}

pub fn module(service: Arc<DownloadService>) -> RegisteredModule {
    let mut module =
        RegisteredModule::new("download", "Download", false, false, commands::commands());
    module.descriptor.provided_services.push("download");
    module.lifecycle = Arc::new(DownloadLifecycle { service });
    module
}
