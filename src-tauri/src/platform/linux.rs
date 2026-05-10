// Linux: XDG Base Directory Specification.
// https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html

use std::path::PathBuf;

use super::{env_path, home_from_env, PlatformPaths};

pub struct LinuxPaths;

impl PlatformPaths for LinuxPaths {
    fn home_dir(&self) -> Option<PathBuf> {
        home_from_env()
    }

    fn temp_dir(&self) -> PathBuf {
        env_path("TMPDIR").unwrap_or_else(|| PathBuf::from("/tmp"))
    }

    fn user_cache_dir(&self) -> Option<PathBuf> {
        env_path("XDG_CACHE_HOME").or_else(|| home_from_env().map(|h| h.join(".cache")))
    }

    fn user_data_dir(&self) -> Option<PathBuf> {
        env_path("XDG_DATA_HOME").or_else(|| home_from_env().map(|h| h.join(".local/share")))
    }

    fn user_config_dir(&self) -> Option<PathBuf> {
        env_path("XDG_CONFIG_HOME").or_else(|| home_from_env().map(|h| h.join(".config")))
    }

    fn trash_dir(&self) -> Option<PathBuf> {
        // XDG trash spec: $XDG_DATA_HOME/Trash
        self.user_data_dir().map(|p| p.join("Trash"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_dir_is_absolute() {
        let p = LinuxPaths.temp_dir();
        assert!(p.is_absolute(), "temp_dir should be absolute, got {p:?}");
    }

    #[test]
    fn cache_dir_resolves_when_home_is_set() {
        if home_from_env().is_some() {
            let p = LinuxPaths.user_cache_dir().expect("cache dir");
            assert!(p.is_absolute(), "cache_dir should be absolute, got {p:?}");
        }
    }

    #[test]
    fn data_and_trash_consistent() {
        if let (Some(data), Some(trash)) = (LinuxPaths.user_data_dir(), LinuxPaths.trash_dir()) {
            assert_eq!(trash, data.join("Trash"));
        }
    }

    #[test]
    fn audit_log_dir_under_data_dir() {
        if let (Some(data), Some(audit)) = (LinuxPaths.user_data_dir(), LinuxPaths.audit_log_dir())
        {
            assert_eq!(audit, data.join("koscleaner"));
        }
    }
}
