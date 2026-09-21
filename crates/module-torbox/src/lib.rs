#![forbid(unsafe_code)]

mod client;
mod commands;

pub use client::TorBoxService;

use bot_core::RegisteredModule;

pub fn module() -> RegisteredModule {
    let mut module = RegisteredModule::new("torbox", "TorBox", false, false, commands::commands());
    module.descriptor.provided_services.push("torbox");
    module
}
