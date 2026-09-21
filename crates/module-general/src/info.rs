use bot_core::serenity;

const INFO_COLOUR: serenity::Colour = serenity::Colour::from_rgb(88, 101, 242);
const ROLE_FIELD_LIMIT: usize = 900;

pub(super) fn embed(title: impl Into<String>) -> serenity::CreateEmbed {
    serenity::CreateEmbed::new()
        .title(title)
        .colour(INFO_COLOUR)
        .footer(serenity::CreateEmbedFooter::new("Momo • Information"))
}

pub(super) fn discord_date(timestamp: serenity::Timestamp) -> String {
    let unix = timestamp.unix_timestamp();
    format!("<t:{unix}:F>\n<t:{unix}:R>")
}

pub(super) fn role_list(names: &[String]) -> String {
    if names.is_empty() {
        return "No roles".to_string();
    }

    let mut shown = Vec::new();
    for name in names {
        let remaining = names.len() - shown.len() - 1;
        let mut candidate = shown.clone();
        candidate.push(name.as_str());
        let mut value = candidate.join(" • ");
        if remaining > 0 {
            value.push_str(&format!(" • +{remaining} more"));
        }
        if value.chars().count() > ROLE_FIELD_LIMIT {
            break;
        }
        shown.push(name.as_str());
    }

    let remaining = names.len() - shown.len();
    let mut value = shown.join(" • ");
    if remaining > 0 {
        if !value.is_empty() {
            value.push_str(" • ");
        }
        value.push_str(&format!("+{remaining} more"));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_role_list_has_a_readable_fallback() {
        assert_eq!(role_list(&[]), "No roles");
    }

    #[test]
    fn role_list_stays_below_the_field_limit_and_reports_omissions() {
        let roles = (0..30)
            .map(|index| format!("Role {index} {}", "x".repeat(80)))
            .collect::<Vec<_>>();
        let rendered = role_list(&roles);

        assert!(rendered.chars().count() <= ROLE_FIELD_LIMIT);
        assert!(rendered.contains(" more"));
    }
}
