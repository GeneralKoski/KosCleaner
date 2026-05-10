// Risoluzione path OS-specific. Ogni cleaner deve passare di qui invece di
// hardcodare path: lo stesso modulo sa cosa significa "cache utente" su Linux,
// macOS e Windows, e i test possono fare mocking sostituendo l'implementazione.

use std::path::PathBuf;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub trait PlatformPaths {
    fn home_dir(&self) -> Option<PathBuf>;
    fn temp_dir(&self) -> PathBuf;
    fn user_cache_dir(&self) -> Option<PathBuf>;
    fn user_data_dir(&self) -> Option<PathBuf>;
    fn user_config_dir(&self) -> Option<PathBuf>;
    fn trash_dir(&self) -> Option<PathBuf>;
    fn audit_log_dir(&self) -> Option<PathBuf> {
        self.user_data_dir().map(|p| p.join("koscleaner"))
    }
}

#[cfg(target_os = "linux")]
pub fn current() -> impl PlatformPaths {
    linux::LinuxPaths
}

#[cfg(target_os = "macos")]
pub fn current() -> impl PlatformPaths {
    macos::MacosPaths
}

#[cfg(target_os = "windows")]
pub fn current() -> impl PlatformPaths {
    windows::WindowsPaths
}

// Helper riusato dalle implementazioni: ritorna $HOME / %USERPROFILE% se settato e non vuoto.
fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

#[allow(deprecated)]
fn home_from_env() -> Option<PathBuf> {
    std::env::home_dir()
}
