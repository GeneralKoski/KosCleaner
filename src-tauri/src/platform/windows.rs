// Windows: Known Folders via env vars (TEMP/TMP, LOCALAPPDATA, APPDATA, USERPROFILE).
// Niente chiamate a SHGetKnownFolderPath qui: env vars coprono il 100% dei casi reali
// e la fonte di verità è la stessa che usa il sistema.

use std::path::PathBuf;

use super::{env_path, home_from_env, PlatformPaths};

pub struct WindowsPaths;

impl PlatformPaths for WindowsPaths {
    fn home_dir(&self) -> Option<PathBuf> {
        env_path("USERPROFILE").or_else(home_from_env)
    }

    fn temp_dir(&self) -> PathBuf {
        env_path("TEMP")
            .or_else(|| env_path("TMP"))
            .unwrap_or_else(std::env::temp_dir)
    }

    fn user_cache_dir(&self) -> Option<PathBuf> {
        env_path("LOCALAPPDATA").or_else(|| self.home_dir().map(|h| h.join(r"AppData\Local")))
    }

    fn user_data_dir(&self) -> Option<PathBuf> {
        env_path("APPDATA").or_else(|| self.home_dir().map(|h| h.join(r"AppData\Roaming")))
    }

    fn user_config_dir(&self) -> Option<PathBuf> {
        // Su Windows non c'è una cartella "config" separata: APPDATA tiene config + state.
        self.user_data_dir()
    }

    fn trash_dir(&self) -> Option<PathBuf> {
        // La Recycle Bin è per-volume e usa il SID dell'utente: la risoluzione vera
        // arriverà nel cleaner trash (Step 11). Qui ritorniamo None per non illudere
        // chi usa questo helper.
        None
    }

    fn audit_log_dir(&self) -> Option<PathBuf> {
        self.user_cache_dir().map(|p| p.join("KosCleaner"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_dir_is_absolute() {
        let p = WindowsPaths.temp_dir();
        assert!(p.is_absolute(), "temp_dir should be absolute, got {p:?}");
    }

    #[test]
    fn cache_dir_resolves_when_env_is_set() {
        if WindowsPaths.home_dir().is_some() {
            let p = WindowsPaths.user_cache_dir().expect("cache dir");
            assert!(p.is_absolute());
        }
    }

    #[test]
    fn config_dir_falls_back_to_data_dir() {
        let cfg = WindowsPaths.user_config_dir();
        let data = WindowsPaths.user_data_dir();
        assert_eq!(cfg, data);
    }

    #[test]
    fn trash_dir_is_none() {
        // Volutamente None su Windows finché non implementiamo il cleaner Recycle Bin.
        assert!(WindowsPaths.trash_dir().is_none());
    }
}
