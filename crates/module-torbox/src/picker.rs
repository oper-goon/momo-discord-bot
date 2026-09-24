use crate::client::TorrentFile;
use anyhow::{Result, anyhow, ensure};
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

pub const PICKER_PAGE_SIZE: usize = 25;
const PICKER_TTL: Duration = Duration::from_secs(5 * 60);
const MAX_PICKERS: usize = 64;
const MAX_PICKER_FILES: usize = 250;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerAction {
    Download,
    Stream,
}

impl PickerAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Download => "download",
            Self::Stream => "stream",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickerEntry {
    pub number: u32,
    pub file: TorrentFile,
}

#[derive(Clone, Debug)]
pub struct PickerPage {
    pub session_id: u64,
    pub action: PickerAction,
    pub page: usize,
    pub pages: usize,
    pub total: usize,
    pub omitted: usize,
    pub download_all_available: bool,
    pub entries: Vec<PickerEntry>,
}

#[derive(Clone, Debug)]
pub struct PickerSelection {
    pub action: PickerAction,
    pub torrent_id: u64,
    pub entry: PickerEntry,
}

#[derive(Clone, Debug)]
pub struct PickerDownloadAll {
    pub torrent_id: u64,
}

#[derive(Debug)]
pub enum PickerAccess<T> {
    Ready(T),
    NotOwner,
    Expired,
}

#[derive(Clone, Copy, Debug)]
pub enum PageDirection {
    Previous,
    Next,
}

struct PickerSession {
    owner_id: u64,
    torrent_id: u64,
    action: PickerAction,
    page: usize,
    omitted: usize,
    download_all_available: bool,
    entries: Vec<PickerEntry>,
    created: Instant,
}

#[derive(Default)]
pub struct PickerStore {
    sessions: Mutex<HashMap<u64, PickerSession>>,
}

impl PickerStore {
    pub fn register(
        &self,
        session_id: u64,
        owner_id: u64,
        torrent_id: u64,
        mut entries: Vec<PickerEntry>,
        action: PickerAction,
        download_all_available: bool,
    ) -> Result<PickerPage> {
        ensure!(!entries.is_empty(), "No safe files are available to select");
        let omitted = entries.len().saturating_sub(MAX_PICKER_FILES);
        entries.truncate(MAX_PICKER_FILES);

        let mut sessions = self.lock()?;
        Self::remove_expired(&mut sessions);
        if sessions.len() >= MAX_PICKERS
            && let Some(oldest) = sessions
                .iter()
                .min_by_key(|(_, session)| session.created)
                .map(|(id, _)| *id)
        {
            sessions.remove(&oldest);
        }
        sessions.insert(
            session_id,
            PickerSession {
                owner_id,
                torrent_id,
                action,
                page: 0,
                omitted,
                download_all_available: action == PickerAction::Download && download_all_available,
                entries,
                created: Instant::now(),
            },
        );
        Ok(Self::page(
            session_id,
            sessions
                .get(&session_id)
                .ok_or_else(|| anyhow!("TorBox picker could not be created"))?,
        ))
    }

    pub fn navigate(
        &self,
        session_id: u64,
        user_id: u64,
        direction: PageDirection,
    ) -> Result<PickerAccess<PickerPage>> {
        let mut sessions = self.lock()?;
        Self::remove_expired(&mut sessions);
        let Some(session) = sessions.get_mut(&session_id) else {
            return Ok(PickerAccess::Expired);
        };
        if session.owner_id != user_id {
            return Ok(PickerAccess::NotOwner);
        }
        let pages = session.entries.len().div_ceil(PICKER_PAGE_SIZE);
        session.page = match direction {
            PageDirection::Previous if session.page == 0 => pages - 1,
            PageDirection::Previous => session.page - 1,
            PageDirection::Next => (session.page + 1) % pages,
        };
        Ok(PickerAccess::Ready(Self::page(session_id, session)))
    }

    pub fn select(
        &self,
        session_id: u64,
        user_id: u64,
        file_number: u32,
    ) -> Result<PickerAccess<PickerSelection>> {
        let mut sessions = self.lock()?;
        Self::remove_expired(&mut sessions);
        let Some(session) = sessions.get(&session_id) else {
            return Ok(PickerAccess::Expired);
        };
        if session.owner_id != user_id {
            return Ok(PickerAccess::NotOwner);
        }
        let start = session.page * PICKER_PAGE_SIZE;
        let end = (start + PICKER_PAGE_SIZE).min(session.entries.len());
        let entry = session.entries[start..end]
            .iter()
            .find(|entry| entry.number == file_number)
            .cloned()
            .ok_or_else(|| anyhow!("That file is not on the current picker page"))?;
        let selection = PickerSelection {
            action: session.action,
            torrent_id: session.torrent_id,
            entry,
        };
        sessions.remove(&session_id);
        Ok(PickerAccess::Ready(selection))
    }

    pub fn select_all(
        &self,
        session_id: u64,
        user_id: u64,
    ) -> Result<PickerAccess<PickerDownloadAll>> {
        let mut sessions = self.lock()?;
        Self::remove_expired(&mut sessions);
        let Some(session) = sessions.get(&session_id) else {
            return Ok(PickerAccess::Expired);
        };
        if session.owner_id != user_id {
            return Ok(PickerAccess::NotOwner);
        }
        ensure!(
            session.action == PickerAction::Download && session.download_all_available,
            "Download all is unavailable for this picker"
        );
        let selection = PickerDownloadAll {
            torrent_id: session.torrent_id,
        };
        sessions.remove(&session_id);
        Ok(PickerAccess::Ready(selection))
    }

    fn page(session_id: u64, session: &PickerSession) -> PickerPage {
        let pages = session.entries.len().div_ceil(PICKER_PAGE_SIZE);
        let start = session.page * PICKER_PAGE_SIZE;
        let end = (start + PICKER_PAGE_SIZE).min(session.entries.len());
        PickerPage {
            session_id,
            action: session.action,
            page: session.page,
            pages,
            total: session.entries.len(),
            omitted: session.omitted,
            download_all_available: session.download_all_available,
            entries: session.entries[start..end].to_vec(),
        }
    }

    fn remove_expired(sessions: &mut HashMap<u64, PickerSession>) {
        sessions.retain(|_, session| session.created.elapsed() < PICKER_TTL);
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, HashMap<u64, PickerSession>>> {
        self.sessions
            .lock()
            .map_err(|_| anyhow!("TorBox picker state is unavailable"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(count: usize) -> Vec<PickerEntry> {
        (0..count)
            .map(|index| PickerEntry {
                number: index as u32 + 1,
                file: TorrentFile {
                    id: index as u64,
                    name: format!("file-{index}"),
                    short_name: None,
                    size: 100,
                    infected: false,
                    mimetype: None,
                },
            })
            .collect()
    }

    #[test]
    fn paginates_and_restricts_sessions_to_the_owner() {
        let store = PickerStore::default();
        let first = store
            .register(10, 20, 30, entries(26), PickerAction::Download, true)
            .unwrap();
        assert_eq!(first.entries.len(), 25);
        assert_eq!(first.pages, 2);

        assert!(matches!(
            store.navigate(10, 99, PageDirection::Next).unwrap(),
            PickerAccess::NotOwner
        ));
        let PickerAccess::Ready(second) = store.navigate(10, 20, PageDirection::Next).unwrap()
        else {
            panic!("expected the second picker page");
        };
        assert_eq!(second.entries.len(), 1);
        assert_eq!(second.entries[0].number, 26);
    }

    #[test]
    fn selection_must_come_from_the_current_page_and_consumes_the_session() {
        let store = PickerStore::default();
        store
            .register(10, 20, 30, entries(26), PickerAction::Stream, false)
            .unwrap();
        assert!(store.select(10, 20, 26).is_err());
        store.navigate(10, 20, PageDirection::Next).unwrap();
        let PickerAccess::Ready(selection) = store.select(10, 20, 26).unwrap() else {
            panic!("expected a file selection");
        };
        assert_eq!(selection.torrent_id, 30);
        assert_eq!(selection.entry.number, 26);
        assert!(matches!(
            store.select(10, 20, 26).unwrap(),
            PickerAccess::Expired
        ));
    }

    #[test]
    fn download_all_is_owner_scoped_and_consumes_the_session() {
        let store = PickerStore::default();
        store
            .register(10, 20, 30, entries(2), PickerAction::Download, true)
            .unwrap();
        assert!(matches!(
            store.select_all(10, 99).unwrap(),
            PickerAccess::NotOwner
        ));
        let PickerAccess::Ready(selection) = store.select_all(10, 20).unwrap() else {
            panic!("expected a bulk download selection");
        };
        assert_eq!(selection.torrent_id, 30);
        assert!(matches!(
            store.select_all(10, 20).unwrap(),
            PickerAccess::Expired
        ));
    }

    #[test]
    fn download_all_is_rejected_when_disabled() {
        let store = PickerStore::default();
        store
            .register(10, 20, 30, entries(2), PickerAction::Download, false)
            .unwrap();
        assert!(store.select_all(10, 20).is_err());
    }
}
