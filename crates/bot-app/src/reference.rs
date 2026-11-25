//! Offline documentation export. No configuration, database, lifecycle or gateway startup.
use super::*;
use serde_json::{Value, json};

pub fn snapshot() -> Result<Value> {
    let config = AppConfig::default();
    let (registry, commands) = compose(
        Arc::new(module_music::MusicService::new(
            &config,
            songbird::Songbird::serenity(),
        )),
        Arc::new(module_download::DownloadService::new(&config)),
        Arc::new(module_image::DuckDuckGoImageService::new()?),
        Arc::new(module_social_embeds::SocialEmbedService::new(&config)?),
    )?;
    fn visit(
        commands: &[Command],
        parent: &str,
        registry: &ModuleRegistry,
        rows: &mut Vec<Value>,
    ) -> Result<()> {
        for command in commands {
            let path = if parent.is_empty() {
                command.name.to_string()
            } else {
                format!("{parent}.{}", command.name)
            };
            let cooldown = command
                .cooldown_config
                .read()
                .map_err(|_| anyhow::anyhow!("Cooldown configuration poisoned"))?;
            let seconds = |duration: Option<Duration>| duration.map(|d| d.as_secs_f64());
            rows.push(json!({
                "path": path, "module": registry.owner(&path)?, "description": command.description,
                "source_function": command.source_code_name, "slash": command.slash_action.is_some(),
                "prefix": command.prefix_action.is_some(), "context_menu": command.context_menu_name,
                "group": !command.subcommands.is_empty(), "guild_only": command.guild_only,
                "dm_only": command.dm_only, "owners_only": command.owners_only, "nsfw_only": command.nsfw_only,
                "required_permissions": command.required_permissions.to_string(),
                "default_member_permissions": command.default_member_permissions.to_string(),
                "required_bot_permissions": command.required_bot_permissions.to_string(),
                "aliases": command.aliases,
                "cooldowns_seconds": {"user": seconds(cooldown.user), "guild": seconds(cooldown.guild), "channel": seconds(cooldown.channel), "member": seconds(cooldown.member), "global": seconds(cooldown.global)},
                "arguments": command.parameters.iter().map(|p| json!({"name": p.name, "required": p.required, "description": p.description, "schema": p.create_as_slash_command_option()})).collect::<Vec<_>>()
            }));
            visit(&command.subcommands, &path, registry, rows)?;
        }
        Ok(())
    }
    let mut rows = Vec::new();
    visit(&commands, "", &registry, &mut rows)?;
    Ok(
        json!({"schema_version": 1, "commands": rows, "config_defaults": config,
        "modules": registry.modules.iter().map(|m| json!({"id": m.descriptor.id, "name": m.descriptor.display_name, "required": m.descriptor.required, "default_enabled": m.descriptor.default_enabled, "intents": format!("{:?}", m.intents), "subscriptions": m.subscriptions})).collect::<Vec<_>>() }),
    )
}

pub fn write(path: &str) -> Result<()> {
    std::fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(&snapshot()?)?),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn export_covers_extensions_context_actions_and_default_cooldown() {
        let snapshot = super::snapshot().unwrap();
        let rows = snapshot["commands"].as_array().unwrap();
        assert!(
            rows.iter()
                .any(|c| c["path"] == "role.info" && c["module"] == "general")
        );
        assert!(rows.iter().any(|c| c["context_menu"] == "Bookmark Message"));
        assert!(
            rows.iter()
                .all(|c| c["cooldowns_seconds"]["user"].as_f64().unwrap() >= 1.0)
        );
        assert_eq!(snapshot["modules"].as_array().unwrap().len(), 11);
    }
}
