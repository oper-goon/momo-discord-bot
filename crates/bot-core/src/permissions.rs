use crate::{Context, Effect, PermissionRule};

pub struct Subject<'a> {
    pub user: u64,
    pub channel: u64,
    pub roles: &'a [u64],
    pub owner: bool,
    pub administrator: bool,
}
#[derive(Debug)]
pub struct Decision {
    pub allowed: bool,
    pub rule_id: Option<i64>,
    pub reason: &'static str,
}

pub fn evaluate(
    rules: &[PermissionRule],
    subject: &Subject<'_>,
    module: &str,
    command: &str,
    admin_bypass: bool,
    default: bool,
) -> Decision {
    if subject.owner || (subject.administrator && admin_bypass) {
        return Decision {
            allowed: true,
            rule_id: None,
            reason: "owner/administrator bypass",
        };
    }
    let mut matches = Vec::new();
    for rule in rules {
        let command_match =
            command == rule.target || command.starts_with(&(rule.target.clone() + "."));
        let module_match = rule.target == module;
        if !command_match && !module_match {
            continue;
        }
        if rule
            .channel_id
            .as_ref()
            .is_some_and(|id| *id != subject.channel.to_string())
        {
            continue;
        }
        let matched = match rule.subject_kind.as_str() {
            "everyone" => true,
            "user" => rule.subject_id.as_deref() == Some(subject.user.to_string().as_str()),
            "role" => subject
                .roles
                .iter()
                .any(|id| rule.subject_id.as_deref() == Some(id.to_string().as_str())),
            "channel" => rule.subject_id.as_deref() == Some(subject.channel.to_string().as_str()),
            _ => false,
        };
        if !matched {
            continue;
        }
        let base = if command_match { 0 } else { 3 };
        let rank = match (rule.subject_kind.as_str(), rule.channel_id.is_some()) {
            ("user", false) => base,
            ("user" | "role", true) => base + 1,
            ("role", false) => base + 2,
            _ => 6,
        };
        matches.push((
            rank,
            std::cmp::Reverse(rule.target.split('.').count()),
            rule.effect == Effect::Allow,
            rule,
        ));
    }
    matches.sort_by_key(|(rank, depth, allow, rule)| (*rank, *depth, *allow, rule.id));
    if let Some((_, _, _, rule)) = matches.first() {
        Decision {
            allowed: rule.effect == Effect::Allow,
            rule_id: Some(rule.id),
            reason: "matching policy rule",
        }
    } else {
        Decision {
            allowed: default,
            rule_id: None,
            reason: "command default",
        }
    }
}

pub async fn decision(ctx: Context<'_>, module: &str, path: &str) -> anyhow::Result<Decision> {
    let Some(guild) = ctx.guild_id() else {
        return Ok(Decision {
            allowed: true,
            rule_id: None,
            reason: "DM command",
        });
    };
    let member = guild.member(ctx.http(), ctx.author().id).await?;
    let roles: Vec<_> = member.roles.iter().map(|r| r.get()).collect();
    let administrator = crate::effective_permissions(ctx)
        .await?
        .is_some_and(|p| p.administrator());
    Ok(evaluate(
        &ctx.data().database.rules(guild.get()).await?,
        &Subject {
            user: ctx.author().id.get(),
            channel: ctx.channel_id().get(),
            roles: &roles,
            owner: ctx
                .data()
                .config
                .discord
                .owners
                .contains(&ctx.author().id.get()),
            administrator,
        },
        module,
        path,
        ctx.data().config.discord.administrator_bypass,
        true,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rule(
        id: i64,
        target: &str,
        kind: &str,
        subject: Option<&str>,
        channel: Option<&str>,
        effect: Effect,
    ) -> PermissionRule {
        PermissionRule {
            id,
            target: target.into(),
            subject_kind: kind.into(),
            subject_id: subject.map(str::to_owned),
            channel_id: channel.map(str::to_owned),
            effect,
        }
    }
    #[test]
    fn precedence_all_roles_and_denies() {
        let subject = Subject {
            user: 1,
            channel: 2,
            roles: &[3, 4],
            owner: false,
            administrator: false,
        };
        let mut rules = vec![
            rule(1, "music", "user", Some("1"), None, Effect::Allow),
            rule(2, "queue.move", "role", Some("3"), None, Effect::Deny),
        ];
        assert!(!evaluate(&rules, &subject, "music", "queue.move", true, true).allowed);
        rules.push(rule(
            3,
            "queue.move",
            "user",
            Some("1"),
            None,
            Effect::Allow,
        ));
        assert!(evaluate(&rules, &subject, "music", "queue.move", true, true).allowed);
        rules.push(rule(4, "queue.move", "user", Some("1"), None, Effect::Deny));
        assert!(!evaluate(&rules, &subject, "music", "queue.move", true, true).allowed);
        assert!(
            evaluate(
                &rules,
                &Subject {
                    owner: true,
                    ..subject
                },
                "music",
                "queue.move",
                true,
                false
            )
            .allowed
        );
    }
    #[test]
    fn child_and_channel_rules() {
        let subject = Subject {
            user: 1,
            channel: 2,
            roles: &[3],
            owner: false,
            administrator: false,
        };
        let rules = vec![
            rule(1, "queue", "role", Some("3"), None, Effect::Deny),
            rule(2, "queue.move", "role", Some("3"), None, Effect::Allow),
        ];
        assert!(evaluate(&rules, &subject, "music", "queue.move", true, true).allowed);
        assert!(!evaluate(&rules, &subject, "music", "queue.remove", true, true).allowed);
    }
}
