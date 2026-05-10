// macOS: Apple File System Programming Guide.
// Cache → ~/Library/Caches, app data → ~/Library/Application Support, ecc.

use std::path::PathBuf;

use super::{env_path, home_from_env, PlatformPaths};

pub struct MacosPaths;

impl PlatformPaths for MacosPaths {
    fn home_dir(&self) -> Option<PathBuf> {
        home_from_env()
    }

    fn temp_dir(&self) -> PathBuf {
        env_path("TMPDIR").unwrap_or_else(|| PathBuf::from("/tmp"))
    }

    fn user_cache_dir(&self) -> Option<PathBuf> {
        home_from_env().map(|h| h.join("Library/Caches"))
    }

    fn user_data_dir(&self) -> Option<PathBuf> {
        home_from_env().map(|h| h.join("Library/Application Support"))
    }

    fn user_config_dir(&self) -> Option<PathBuf> {
        home_from_env().map(|h| h.join("Library/Preferences"))
    }

    fn trash_dir(&self) -> Option<PathBuf> {
        home_from_env().map(|h| h.join(".Trash"))
    }

    fn audit_log_dir(&self) -> Option<PathBuf> {
        home_from_env().map(|h| h.join("Library/Logs/KosCleaner"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_dir_is_absolute() {
        let p = MacosPaths.temp_dir();
        assert!(p.is_absolute(), "temp_dir should be absolute, got {p:?}");
    }

    #[test]
    fn cache_dir_under_library() {
        if let Some(cache) = MacosPaths.user_cache_dir() {
            assert!(cache.ends_with("Library/Caches"));
        }
    }

    #[test]
    fn data_dir_under_application_support() {
        if let Some(data) = MacosPaths.user_data_dir() {
            assert!(data.ends_with("Library/Application Support"));
        }
    }

    #[test]
    fn trash_dir_is_dot_trash() {
        if let Some(trash) = MacosPaths.trash_dir() {
            assert!(trash.ends_with(".Trash"));
        }
    }

    #[test]
    fn audit_log_dir_under_logs() {
        if let Some(audit) = MacosPaths.audit_log_dir() {
            assert!(audit.ends_with("Library/Logs/KosCleaner"));
        }
    }
}
