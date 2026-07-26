use std::collections::HashMap;
use std::sync::Mutex;

use tauri::{AppHandle, State};

/// Per-profile unread counts + helpers to refresh tray.
pub struct UnreadState {
    pub counts: Mutex<HashMap<String, u32>>,
}

impl UnreadState {
    pub fn new() -> Self {
        Self {
            counts: Mutex::new(HashMap::new()),
        }
    }

    pub fn total(&self) -> u32 {
        self.counts
            .lock()
            .map(|m| m.values().fold(0u32, |acc, v| acc.saturating_add(*v)))
            .unwrap_or(0)
    }

    pub fn set(&self, profile: &str, count: u32) {
        if let Ok(mut m) = self.counts.lock() {
            if count == 0 {
                m.remove(profile);
            } else {
                m.insert(profile.to_string(), count);
            }
        }
    }

    pub fn rename_key(&self, from: &str, to: &str) {
        if let Ok(mut m) = self.counts.lock() {
            if let Some(v) = m.remove(from) {
                m.insert(to.to_string(), v);
            }
        }
    }

    pub fn remove_key(&self, profile: &str) {
        if let Ok(mut m) = self.counts.lock() {
            m.remove(profile);
        }
    }
}

#[tauri::command]
pub fn set_unread_count(
    app: AppHandle,
    state: State<UnreadState>,
    profile: String,
    count: u32,
) -> Result<(), String> {
    state.set(&profile, count);
    crate::tray::refresh_unread(&app, state.total());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_sums_profiles() {
        let s = UnreadState::new();
        s.set("a", 2);
        s.set("b", 3);
        assert_eq!(s.total(), 5);
    }

    #[test]
    fn total_saturates_instead_of_overflowing() {
        let s = UnreadState::new();
        s.set("a", u32::MAX);
        s.set("b", u32::MAX);
        assert_eq!(s.total(), u32::MAX);
    }

    #[test]
    fn zero_removes_entry_and_rename_moves_count() {
        let s = UnreadState::new();
        s.set("a", 4);
        s.rename_key("a", "b");
        assert_eq!(s.total(), 4);
        s.set("b", 0);
        assert_eq!(s.total(), 0);
    }
}
