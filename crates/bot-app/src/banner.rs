use std::io::{self, IsTerminal, Write};

// TAAG Ghost Silver: background colors are part of the original artwork.
const LOGO: &str = "   ▄▄▄▄▄     ▄▄▄▄▄     ▄▄▄▄▄     ▄▄▄▄▄  \n ▄█▀█▀██▄▄ ▄▀▀ ▀██▄▄ ▄█▀█▀██▄▄ ▄▀▀ ▀██▄▄\n▐█▌▌▐▌▌███ ██    ███▐█▌▌▐▌▌███ ██    ███\n▐███ ▀▐█▌▌▐█▄▄ ▄███ ▐███ ▀▐█▌▌▐█▄▄ ▄███ \n▐██▌▌ ███  ▀███▀▀▀  ▐██▌▌ ███  ▀███▀▀▀  ";
const PALETTE: &str = "SSSWSWWSSSSSSWWWWSSSSSSWSWWSSSSSSWWWWSSS\nSWWBWBWWBSSWBSSWWWBSSWWBWBWWBSSWBSSWWWBS\nWWBSWBSWWSSWSSSSSWWSWWBSWBSWWSSWSSSSSWWS\nWWWSSSWWBSWWBSSWWWSSWWWSSSWWBSWWBSSWWWSS\nWWWBSSWWSSSWWWWBWSSSWWWBSSWWSSSWWWWBWSSS";

pub struct StartupInfo<'a> {
    pub bot: &'a str,
    pub servers: usize,
    pub modules: usize,
    pub commands: usize,
    pub command_mode: &'a str,
    pub prefix_commands: bool,
    pub duration: std::time::Duration,
    pub updates: &'a str,
}

pub fn print(info: StartupInfo<'_>) {
    let stdout = io::stdout();
    let color = stdout.is_terminal()
        && std::env::var_os("NO_COLOR").is_none()
        && std::env::var("TERM").as_deref() != Ok("dumb");
    let mut output = String::from("\n");
    for (character, shade) in LOGO.chars().zip(PALETTE.chars()) {
        if color {
            output.push_str(match shade {
                'W' => "\x1b[97;40m",
                'B' => "\x1b[97;100m",
                'S' => "\x1b[37;40m",
                _ => "\x1b[0m",
            });
        }
        output.push(character);
    }
    if color {
        output.push_str("\x1b[0m");
    }
    // Keep Discord names on one line and prevent terminal control injection.
    let bot: String = info.bot.chars().filter(|c| !c.is_control()).collect();
    output.push_str(&format!("\n\n  {bot} • Self-hosted Discord bot\n\n"));
    if color {
        output.push_str("\x1b[36m");
    }
    let rows = [
        ("Version", env!("CARGO_PKG_VERSION").to_owned()),
        ("Status", "Ready".to_owned()),
        (
            "Startup duration",
            format!("{:.2}s", info.duration.as_secs_f64()),
        ),
        ("Updates", info.updates.to_owned()),
        ("Servers", info.servers.to_string()),
        ("Loaded modules", info.modules.to_string()),
        ("Top-level commands", info.commands.to_string()),
        ("Command registration", info.command_mode.to_owned()),
        (
            "Prefix commands",
            if info.prefix_commands {
                "Enabled"
            } else {
                "Disabled"
            }
            .to_owned(),
        ),
    ];
    let width = rows
        .iter()
        .map(|(_, value)| 24 + value.chars().count())
        .max()
        .unwrap_or(52)
        .max(52);
    output.push_str(&format!("  ╭{}╮\n", "─".repeat(width)));
    for (label, value) in rows {
        let row = format!("  {label:<22}{value}");
        output.push_str(&format!("  │{row:<width$}│\n"));
    }
    output.push_str(&format!("  ╰{}╯\n", "─".repeat(width)));
    if color {
        output.push_str("\x1b[0m");
    }
    output.push_str("\n  Created by goon[at]internetabuse[dot]net\n\n");
    // A closed output pipe must not prevent the bot from starting.
    let _ = stdout.lock().write_all(output.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silver_palette_matches_every_logo_character() {
        assert_eq!(LOGO.chars().count(), PALETTE.chars().count());
        for (character, shade) in LOGO.chars().zip(PALETTE.chars()) {
            assert_eq!(character == '\n', shade == '\n');
            assert!(matches!(shade, 'W' | 'B' | 'S' | '\n'));
        }
    }
}
