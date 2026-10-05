//! # Canonical Installed Resource Resolution (§5, §8)
//!
//! Resolves runtime paths (worker binaries, model files, manifests, licenses)
//! relative to the installed application executable rather than developer paths.
//!
//! ## Invariants
//!
//! - Never uses hardcoded developer paths (e.g. `C:\Users\...`, `target/debug`).
//! - Never assumes the current working directory is the installation directory.
//! - Resolves relative to `std::env::current_exe()` with fallback to `%LOCALAPPDATA%`.

use std::path::{Path, PathBuf};

/// Manager for resolving application resources dynamically at runtime.
#[derive(Debug, Clone)]
pub struct AppResourcePaths {
    /// Base application directory (containing `FlowDictate.exe`).
    app_root: PathBuf,
}

impl Default for AppResourcePaths {
    fn default() -> Self {
        Self::from_current_exe()
    }
}

impl AppResourcePaths {
    /// Initialize paths using the directory of the running executable.
    pub fn from_current_exe() -> Self {
        let app_root = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| PathBuf::from("."));

        Self { app_root }
    }

    /// Initialize with an explicit application root directory (useful for tests).
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            app_root: root.as_ref().to_path_buf(),
        }
    }

    /// Path to the application root directory.
    pub fn app_root(&self) -> &Path {
        &self.app_root
    }

    /// Resolves path to a worker executable in `bin/` or adjacent to the app executable.
    pub fn find_worker_executable(&self, worker_name: &str) -> Option<PathBuf> {
        let exe_name = if worker_name.ends_with(".exe") {
            worker_name.to_string()
        } else {
            format!("{worker_name}.exe")
        };

        // 1. Check in [app_root]/bin/
        let in_bin = self.app_root.join("bin").join(&exe_name);
        if in_bin.is_file() {
            return Some(in_bin);
        }

        // 2. Check directly in [app_root]/
        let in_root = self.app_root.join(&exe_name);
        if in_root.is_file() {
            return Some(in_root);
        }

        // 3. Fallback for cargo workspace target directories during dev
        let in_target = self.app_root.join("target").join("release").join(&exe_name);
        if in_target.is_file() {
            return Some(in_target);
        }

        // Return preferred production path even if not yet on disk
        Some(in_bin)
    }

    /// Resolves path to the models directory (`[app_root]/models/` or `%LOCALAPPDATA%/FlowDictate/models/`).
    pub fn models_dir(&self) -> PathBuf {
        let bundled = self.app_root.join("models");
        if bundled.is_dir() {
            return bundled;
        }

        // Check Windows LocalAppData
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let user_models = PathBuf::from(local_app_data)
                .join("FlowDictate")
                .join("models");
            if user_models.is_dir() {
                return user_models;
            }
        }

        bundled
    }

    /// Resolves path to the models manifest (`[app_root]/manifests/models.json` or `[models_dir]/manifest.json`).
    pub fn manifest_file(&self) -> PathBuf {
        let in_manifests = self.app_root.join("manifests").join("models.json");
        if in_manifests.is_file() {
            return in_manifests;
        }

        let in_models = self.models_dir().join("manifest.json");
        if in_models.is_file() {
            return in_models;
        }

        in_manifests
    }

    /// Resolves path to native DLL directory (`[app_root]/bin/` or `[app_root]/`).
    pub fn dll_dir(&self) -> PathBuf {
        let in_bin = self.app_root.join("bin");
        if in_bin.is_dir() {
            in_bin
        } else {
            self.app_root.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_paths_resolution() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppResourcePaths::new(temp.path());

        assert_eq!(paths.app_root(), temp.path());
        assert_eq!(
            paths.find_worker_executable("flowdictate-asr-worker"),
            Some(temp.path().join("bin").join("flowdictate-asr-worker.exe"))
        );
        assert_eq!(paths.models_dir(), temp.path().join("models"));
    }
}
