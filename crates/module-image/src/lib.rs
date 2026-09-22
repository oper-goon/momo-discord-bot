#![forbid(unsafe_code)]

mod client;
mod commands;

pub use client::DuckDuckGoImageService;

use anyhow::Result;
use async_trait::async_trait;
use bot_core::{BotData, EventHandler, RegisteredModule, serenity};
use std::sync::Arc;

struct ImageEvents {
    service: Arc<DuckDuckGoImageService>,
}

#[async_trait]
impl EventHandler for ImageEvents {
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
            return commands::component(&ctx, component, &data, &self.service).await;
        }
        Ok(())
    }
}

pub fn module(service: Arc<DuckDuckGoImageService>) -> RegisteredModule {
    let mut module =
        RegisteredModule::new("image", "Image Search", false, false, commands::commands());
    module.descriptor.provided_services.push("image-search");
    module.subscriptions.push("image_component");
    module.handler = Some(Arc::new(ImageEvents { service }));
    module
}
