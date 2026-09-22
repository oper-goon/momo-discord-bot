#![forbid(unsafe_code)]

mod client;
mod commands;

pub use client::DuckDuckGoImageService;

use bot_core::RegisteredModule;

pub fn module() -> RegisteredModule {
    let mut module =
        RegisteredModule::new("image", "Image Search", false, false, commands::commands());
    module.descriptor.provided_services.push("image-search");
    module
}
