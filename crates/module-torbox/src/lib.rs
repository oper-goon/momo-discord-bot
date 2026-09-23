#![forbid(unsafe_code)]

mod client;
mod commands;
mod picker;

pub use client::TorBoxService;

use anyhow::Result;
use async_trait::async_trait;
use bot_core::{BotData, EventHandler, RegisteredModule, serenity};
use std::sync::Arc;

struct TorBoxEvents;

#[async_trait]
impl EventHandler for TorBoxEvents {
    async fn handle(
        &self,
        ctx: serenity::Context,
        event: serenity::FullEvent,
        data: BotData,
    ) -> Result<()> {
        if let serenity::FullEvent::InteractionCreate {
            interaction: serenity::Interaction::Component(component),
        } = &event
        {
            let service = data.services.get::<TorBoxService>()?;
            return commands::component(&ctx, component, &data, &service).await;
        }
        Ok(())
    }
}

pub fn module() -> RegisteredModule {
    let mut module = RegisteredModule::new("torbox", "TorBox", false, false, commands::commands());
    module.descriptor.provided_services.push("torbox");
    module.subscriptions.push("torbox_component");
    module.handler = Some(Arc::new(TorBoxEvents));
    module
}
