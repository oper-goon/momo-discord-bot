//! Conservative shared matching. Missing metadata never overrides a mismatch.
use std::collections::HashSet;

#[derive(Clone, Debug, Default)]
pub struct Metadata {
    pub artist: Option<String>,
    pub title: String,
    pub album: Option<String>,
    pub duration_ms: Option<u64>,
    pub format: Option<String>,
    pub bitrate: Option<u64>,
}

pub fn audio_extension(name: &str) -> Option<&str> {
    let ext = name.rsplit('.').next()?;
    [
        "mp3", "flac", "m4a", "aac", "ogg", "opus", "wav", "aiff", "aif", "alac",
    ]
    .into_iter()
    .find(|allowed| ext.eq_ignore_ascii_case(allowed))
}

fn words(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .replace("official music video", "")
        .replace("official lyric video", "")
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| {
            !s.is_empty() && !["official", "audio", "video", "lyrics", "hd", "hq"].contains(s)
        })
        .map(str::to_owned)
        .collect()
}

/// Strip release bookkeeping from standalone audio filenames, preserving identity/version words.
pub fn audio_filename_title(filename: &str) -> String {
    let mut title = filename.trim();
    if let Some(rest) = title.strip_prefix('(').or_else(|| title.strip_prefix('['))
        && let Some(end) = rest.find([')', ']'])
        && (1..=3).contains(&end)
        && rest[..end].bytes().all(|b| b.is_ascii_digit())
    {
        title = rest[end + 1..].trim_start();
    }
    let digits = title.bytes().take_while(u8::is_ascii_digit).count();
    if (1..=3).contains(&digits) {
        let rest = &title[digits..];
        if let Some(rest) = rest
            .strip_prefix('-')
            .or_else(|| rest.strip_prefix('.'))
            .or_else(|| rest.strip_prefix('_'))
        {
            title = rest.trim_start();
        }
    }
    if let Some((name, suffix)) = title.rsplit_once('-')
        && suffix.len() == 8
        && suffix.bytes().all(|b| b.is_ascii_hexdigit())
        && suffix.bytes().any(|b| b.is_ascii_digit())
        && suffix.bytes().any(|b| b.is_ascii_alphabetic())
    {
        title = name;
    }
    title
        .replace('_', " ")
        .replace('-', " - ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

impl Metadata {
    pub fn from_title(title: &str, duration_ms: Option<u64>) -> Self {
        let (artist, title) = title
            .split_once(" - ")
            .map(|(a, t)| (Some(a.trim().to_owned()), t.trim()))
            .unwrap_or((None, title));
        Self {
            artist,
            title: title.to_owned(),
            duration_ms,
            ..Self::default()
        }
    }
    pub fn query(&self) -> String {
        format!("{} {}", self.artist.as_deref().unwrap_or(""), self.title)
            .trim()
            .to_owned()
    }
}

fn has_unrequested_version(requested: &HashSet<String>, actual: &HashSet<String>) -> bool {
    [
        "live",
        "cover",
        "karaoke",
        "instrumental",
        "remix",
        "mix",
        "slowed",
        "sped",
        "speed",
        "nightcore",
        "covers",
        "remixed",
        "remixes",
        "instrumentals",
        "acoustic",
        "demo",
        "extended",
        "reverb",
    ]
    .into_iter()
    .any(|version| actual.contains(version) && !requested.contains(version))
}

/// Initial natural-language discovery may contain only part of a song's title.
/// Require the complete identified artist and every requested title word; an
/// explicit Artist - Title or known recording metadata remains a strict match.
/// Never use this function to replace an already selected track on fallback.
pub fn query_confidence(wanted: &Metadata, candidate: &Metadata) -> f64 {
    let strict = confidence(wanted, candidate);
    if wanted.artist.is_some() || wanted.duration_ms.is_some() || wanted.album.is_some() {
        return strict;
    }
    let Some(artist) = &candidate.artist else {
        // Without a distinct artist field, extra words cannot safely be
        // identified as title completion rather than a different artist.
        return strict;
    };
    let requested = words(&wanted.title);
    let artist = words(artist);
    let title = words(&candidate.title);
    let title_query: HashSet<_> = requested.difference(&artist).cloned().collect();
    if artist.is_empty()
        || !artist.is_subset(&requested)
        || title_query.is_empty()
        || !title_query.is_subset(&title)
        || has_unrequested_version(&requested, &words(&candidate.query()))
    {
        return strict;
    }
    // All query identity words matched. Do not penalize the remaining title
    // words as though the user supplied a complete canonical recording title.
    // Reuse strict self-validation to retain format/metadata validity checks.
    strict.max(confidence(candidate, candidate))
}

/// Preserve a YouTube creator attribution only when the user explicitly named
/// that entire creator and matching title words. A channel is not a verified
/// recording artist; this enrichment is restricted to initial natural queries.
pub fn creator_attributed_title(wanted: &Metadata, title: &str, creator: &str) -> Option<String> {
    if wanted.artist.is_some()
        || wanted.duration_ms.is_some()
        || wanted.album.is_some()
        || Metadata::from_title(title, None).artist.is_some()
    {
        return None;
    }
    let requested = words(&wanted.title);
    let creator_words = words(creator);
    if creator_words.is_empty()
        || !creator_words.is_subset(&requested)
        || requested.difference(&creator_words).next().is_none()
    {
        return None;
    }
    let candidate = Metadata {
        artist: Some(creator.to_owned()),
        title: title.to_owned(),
        ..Metadata::default()
    };
    (query_confidence(wanted, &candidate) == 1.0)
        .then(|| format!("{} - {}", creator.trim(), title.trim()))
}

pub fn confidence(wanted: &Metadata, candidate: &Metadata) -> f64 {
    let requested = words(&wanted.query());
    let actual = words(&candidate.query());
    if requested.is_empty() || actual.is_empty() || has_unrequested_version(&requested, &actual) {
        return 0.0;
    }
    let title = words(&wanted.title);
    if title.is_empty() || !title.is_subset(&actual) {
        return 0.0;
    }
    if let Some(artist) = &wanted.artist
        && !words(artist).is_subset(&actual)
    {
        return 0.0;
    }
    let mut score = requested.intersection(&actual).count() as f64 / requested.len() as f64;
    // A shared short title is insufficient: "Song" must not match "Song Two".
    let extras = actual.difference(&requested).count();
    score -= (extras as f64 * 0.18).min(0.6);
    if let (Some(a), Some(b)) = (wanted.duration_ms, candidate.duration_ms) {
        let difference = a.abs_diff(b);
        if difference > 10_000.max(a / 20) {
            return 0.0;
        }
        score -= difference as f64 / a.max(1) as f64;
    }
    if let (Some(a), Some(b)) = (&wanted.album, &candidate.album)
        && words(a) != words(b)
    {
        score -= 0.15;
    }
    if candidate
        .format
        .as_deref()
        .is_some_and(|f| audio_extension(f).is_none())
    {
        return 0.0;
    }
    score.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn natural_query_completes_title_only_for_the_identified_artist() {
        let query = Metadata::from_title("tyler the creator sugar", None);
        let official = Metadata::from_title(
            "Tyler, The Creator - Sugar On My Tongue (Official Video)",
            Some(183_000),
        );
        assert!(confidence(&query, &official) < 0.85);
        assert_eq!(query_confidence(&query, &official), 1.0);
        for title in [
            "Other Artist - Sugar On My Tongue",
            "Tyler, The Creator - Sugar On My Tongue (Live)",
            "Tyler, The Creator - Sugar On My Tongue (Cover)",
            "Tyler, The Creator - Sugar On My Tongue (Remix)",
            "Tyler, The Creator - Sugar On My Tongue (Freak Mix)",
            "Tyler, The Creator - Sugar On My Tongue (Slowed)",
            "Tyler, The Creator - Another Song",
            "Sugar On My Tongue",
        ] {
            assert!(query_confidence(&query, &Metadata::from_title(title, None)) < 0.85);
        }
        assert!(
            query_confidence(
                &Metadata::from_title("tyler the creator sugar unrelated", None),
                &official
            ) < 0.85
        );
        assert!(
            query_confidence(
                &Metadata::from_title("Tyler, The Creator - Sugar", None),
                &official
            ) < 0.85
        );
        let full = Metadata::from_title("Artist - Song", Some(200_000));
        let expanded = Metadata::from_title("Artist - Song Two", Some(200_000));
        assert!(confidence(&full, &expanded) < 0.85);
        assert!(query_confidence(&full, &expanded) < 0.85);
        assert!(
            query_confidence(
                &Metadata::from_title("Artist Song", Some(200_000)),
                &expanded
            ) < 0.85
        );
    }
    #[test]
    fn conway_sarah_query_matches_official_video_and_release_filenames() {
        let wanted = Metadata::from_title("Conway the machine sarah", None);
        assert_eq!(
            confidence(
                &wanted,
                &Metadata::from_title(
                    "Conway the Machine - Sarah! (Official Music Video)",
                    Some(218_000)
                )
            ),
            1.0
        );
        for name in [
            "04-conway_the_machine-sarah-0f64f40b",
            "04-conway_the_machine-sarah-888b1943",
            "04-conway_the_machine-sarah-9754c7d6",
        ] {
            let title = audio_filename_title(name);
            assert_eq!(title, "conway the machine - sarah");
            assert_eq!(
                confidence(&wanted, &Metadata::from_title(&title, Some(211_000))),
                1.0
            );
        }
        assert!(
            confidence(
                &wanted,
                &Metadata::from_title(
                    &audio_filename_title("04-conway_the_machine-sarah-remix-0f64f40b"),
                    None
                )
            ) < 0.85
        );
        assert!(
            confidence(
                &wanted,
                &Metadata::from_title(
                    &audio_filename_title("04-conway_the_machine-sarah-two-0f64f40b"),
                    None
                )
            ) < 0.85
        );
        assert_eq!(audio_filename_title("1979"), "1979");
        assert_eq!(audio_filename_title("99 Problems"), "99 Problems");
        assert_eq!(
            audio_filename_title("(08) [Rick Astley] - Never Gonna Give You Up"),
            "[Rick Astley] - Never Gonna Give You Up"
        );
    }
    #[test]
    fn matching_rejects_versions_artist_and_duration_mismatches() {
        let wanted = Metadata::from_title("Artist - Song", Some(200_000));
        for title in [
            "Other - Song",
            "Artist - Song live",
            "Artist - Song karaoke",
            "Artist - Song remix",
            "Artist - Song slowed",
            "Artist - Song cover",
        ] {
            assert!(confidence(&wanted, &Metadata::from_title(title, Some(200_000))) < 0.85);
        }
        assert_eq!(
            confidence(
                &wanted,
                &Metadata::from_title("Artist - Song", Some(300_000))
            ),
            0.0
        );
        assert_eq!(confidence(&wanted, &wanted), 1.0);
        assert!(confidence(&wanted, &Metadata::from_title("Artist - Song Two", None)) < 0.85);
        assert_eq!(
            confidence(
                &wanted,
                &Metadata::from_title("Artist Song official audio", Some(200_000))
            ),
            1.0
        );
        let live = Metadata::from_title("Artist - Song live", None);
        assert_eq!(confidence(&live, &live), 1.0);
    }
}
