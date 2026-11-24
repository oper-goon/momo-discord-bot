use crate::{Context, effective_permissions};

pub fn allows(channels: &[String], channel: u64, parent: Option<u64>, administrator: bool) -> bool {
    administrator
        || channels.is_empty()
        || channels.iter().any(|id| {
            id == &channel.to_string() || parent.is_some_and(|parent| id == &parent.to_string())
        })
}

/// Return the allowed channels when this invocation is blocked.
pub async fn blocked_channels(
    ctx: Context<'_>,
    module: &str,
) -> anyhow::Result<Option<Vec<String>>> {
    let Some(guild) = ctx.guild_id() else {
        return Ok(None);
    };
    if module == "core" {
        return Ok(None);
    }
    let channels = ctx
        .data()
        .database
        .module_channels(guild.get(), module)
        .await?;
    if channels.is_empty() {
        return Ok(None);
    }
    let administrator = effective_permissions(ctx)
        .await?
        .is_some_and(|p| p.administrator());
    if administrator {
        return Ok(None);
    }
    let channel = ctx
        .channel_id()
        .to_channel(ctx.http())
        .await?
        .guild()
        .ok_or_else(|| anyhow::anyhow!("Server channel required"))?;
    let parent = matches!(
        channel.kind,
        crate::serenity::ChannelType::PublicThread
            | crate::serenity::ChannelType::PrivateThread
            | crate::serenity::ChannelType::NewsThread
    )
    .then_some(channel.parent_id)
    .flatten()
    .map(|id| id.get());
    Ok((!allows(&channels, channel.id.get(), parent, administrator)).then_some(channels))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unrestricted_exact_thread_and_admin_access() {
        let channels = vec!["10".to_owned()];
        assert!(allows(&[], 20, None, false));
        assert!(allows(&channels, 10, None, false));
        assert!(allows(&channels, 20, Some(10), false));
        assert!(!allows(&channels, 20, Some(30), false));
        assert!(allows(&channels, 20, None, true));
    }
}
