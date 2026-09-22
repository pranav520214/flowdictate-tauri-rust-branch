//! # Global Hotkey Management
//!
//! Registers and handles system-wide hotkeys for push-to-talk dictation.

use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyManager,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HotkeyError {
    #[error("failed to register hotkey: {0}")]
    RegistrationFailed(String),
    #[error("hotkey manager unavailable: {0}")]
    ManagerUnavailable(String),
}

/// Manages system-wide hotkeys for FlowDictate.
pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    hotkey: HotKey,
}

impl HotkeyManager {
    /// Initializes hotkey manager with default hotkey (Ctrl + Shift + Space).
    pub fn new() -> Result<Self, HotkeyError> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| HotkeyError::ManagerUnavailable(format!("{e:?}")))?;

        // Default: Ctrl + Shift + Space
        let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
        manager
            .register(hotkey)
            .map_err(|e| HotkeyError::RegistrationFailed(format!("{e:?}")))?;

        Ok(Self { manager, hotkey })
    }

    /// The HotKey ID to match against GlobalHotKeyEvent.
    pub fn hotkey_id(&self) -> u32 {
        self.hotkey.id()
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        let _ = self.manager.unregister(self.hotkey);
    }
}
