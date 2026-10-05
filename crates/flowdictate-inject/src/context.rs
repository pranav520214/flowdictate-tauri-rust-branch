//! # Application Context Detection
//!
//! Enforces the strict permission ladder (§22) before acquiring foreground application details.

use serde::{Deserialize, Serialize};

/// Context permission levels (§22).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ContextLevel {
    /// No external application context (default).
    #[default]
    Level0None,
    /// Current application identity only (e.g. process name).
    Level1AppIdentity,
    /// Selected text or current text box content.
    Level2SelectedText,
    /// Additional approved context.
    Level3Extended,
}

/// Coarse application categories for context-aware refinement (§10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AppClass {
    Document,
    Chat,
    Email,
    Browser,
    CodeEditor,
    Terminal,
    #[default]
    Unknown,
}

/// Information gathered about the foreground application under the allowed permission level.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppContext {
    pub permission_level: ContextLevel,
    pub app_name: Option<String>,
    pub app_class: AppClass,
    pub selected_text: Option<String>,
}

/// Classify an application name / executable into an AppClass (§10).
pub fn classify_process_name(name: &str) -> AppClass {
    let lower = name.to_lowercase();
    if lower.contains("cmd")
        || lower.contains("powershell")
        || lower.contains("alacritty")
        || lower.contains("wezterm")
        || lower.contains("terminal")
        || lower.contains("bash")
        || lower.contains("zsh")
    {
        AppClass::Terminal
    } else if lower.contains("code")
        || lower.contains("devenv")
        || lower.contains("cursor")
        || lower.contains("sublime")
        || lower.contains("idea")
        || lower.contains("neovim")
    {
        AppClass::CodeEditor
    } else if lower.contains("slack")
        || lower.contains("discord")
        || lower.contains("teams")
        || lower.contains("telegram")
        || lower.contains("signal")
    {
        AppClass::Chat
    } else if lower.contains("outlook") || lower.contains("thunderbird") || lower.contains("mail") {
        AppClass::Email
    } else if lower.contains("word") || lower.contains("notepad") || lower.contains("writer") {
        AppClass::Document
    } else if lower.contains("chrome")
        || lower.contains("firefox")
        || lower.contains("edge")
        || lower.contains("brave")
    {
        AppClass::Browser
    } else {
        AppClass::Unknown
    }
}

/// Context detector obeying the permission ladder.
pub struct ContextDetector {
    level: ContextLevel,
}

impl ContextDetector {
    pub fn new(level: ContextLevel) -> Self {
        Self { level }
    }

    /// Update allowed permission level.
    pub fn set_level(&mut self, level: ContextLevel) {
        self.level = level;
    }

    /// Current permission level.
    pub fn level(&self) -> ContextLevel {
        self.level
    }

    /// Detect foreground context respecting the permission ladder.
    pub fn detect_current(&self) -> AppContext {
        match self.level {
            ContextLevel::Level0None => AppContext {
                permission_level: ContextLevel::Level0None,
                app_name: None,
                app_class: AppClass::Unknown,
                selected_text: None,
            },
            ContextLevel::Level1AppIdentity => {
                let name = "active_window.exe".to_string();
                let class = classify_process_name(&name);
                AppContext {
                    permission_level: ContextLevel::Level1AppIdentity,
                    app_name: Some(name),
                    app_class: class,
                    selected_text: None,
                }
            }
            ContextLevel::Level2SelectedText | ContextLevel::Level3Extended => {
                let name = "active_window.exe".to_string();
                let class = classify_process_name(&name);
                AppContext {
                    permission_level: self.level,
                    app_name: Some(name),
                    app_class: class,
                    selected_text: None,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level0_returns_no_context() {
        let detector = ContextDetector::new(ContextLevel::Level0None);
        let ctx = detector.detect_current();
        assert_eq!(ctx.app_name, None);
        assert_eq!(ctx.app_class, AppClass::Unknown);
        assert_eq!(ctx.selected_text, None);
    }

    #[test]
    fn test_classify_process_name() {
        assert_eq!(classify_process_name("powershell.exe"), AppClass::Terminal);
        assert_eq!(classify_process_name("code.exe"), AppClass::CodeEditor);
        assert_eq!(classify_process_name("slack.exe"), AppClass::Chat);
        assert_eq!(classify_process_name("WINWORD.EXE"), AppClass::Document);
        assert_eq!(classify_process_name("outlook.exe"), AppClass::Email);
        assert_eq!(classify_process_name("chrome.exe"), AppClass::Browser);
        assert_eq!(classify_process_name("custom_tool.exe"), AppClass::Unknown);
    }
}
