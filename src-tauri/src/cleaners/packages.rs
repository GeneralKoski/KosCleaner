// Package manager cache. Solo path utente: niente sudo, niente apt/dnf/pacman a
// livello di sistema (lo Step 13/successivi affronteranno l'elevazione di
// privilegi). Cache come pip/npm/Homebrew sono interamente in user-space e
// rigenerabili.

use std::path::PathBuf;

use crate::platform::PlatformPaths;

use super::Cleaner;

pub struct PipCacheCleaner;
impl Cleaner for PipCacheCleaner {
    fn id(&self) -> &'static str {
        "packages.pip.cache"
    }
    fn category(&self) -> &'static str {
        "Packages"
    }
    fn name(&self) -> &'static str {
        "pip cache"
    }
    fn description(&self) -> &'static str {
        "Wheels and HTTP responses cached by pip. Will be re-downloaded on next install."
    }
    fn roots(&self, paths: &dyn PlatformPaths) -> Vec<PathBuf> {
        let Some(cache) = paths.user_cache_dir() else {
            return Vec::new();
        };
        #[cfg(windows)]
        let p = cache.join("pip").join("Cache");
        #[cfg(not(windows))]
        let p = cache.join("pip");
        if p.is_dir() {
            vec![p]
        } else {
            Vec::new()
        }
    }
}

pub struct NpmCacheCleaner;
impl Cleaner for NpmCacheCleaner {
    fn id(&self) -> &'static str {
        "packages.npm.cache"
    }
    fn category(&self) -> &'static str {
        "Packages"
    }
    fn name(&self) -> &'static str {
        "npm cache"
    }
    fn description(&self) -> &'static str {
        "Tarball and metadata cache for npm. Re-downloaded on demand."
    }
    fn roots(&self, paths: &dyn PlatformPaths) -> Vec<PathBuf> {
        // Si privilegia _cacache (sub-cartella moderna di npm) per non includere
        // anche metadata files non-cache come logs o anonymous-cli-metrics.
        // Fallback su ~/.npm intero solo se _cacache non esiste.
        let mut roots: Vec<PathBuf> = Vec::new();
        if let Some(home) = paths.home_dir() {
            let cacache = home.join(".npm").join("_cacache");
            let dot_npm = home.join(".npm");
            if cacache.is_dir() {
                roots.push(cacache);
            } else if dot_npm.is_dir() {
                roots.push(dot_npm);
            }
        }
        if let Some(data) = paths.user_data_dir() {
            // Windows: %APPDATA%\npm-cache
            let win = data.join("npm-cache");
            if win.is_dir() {
                roots.push(win);
            }
        }
        roots
    }
}

pub struct HomebrewCacheCleaner;
impl Cleaner for HomebrewCacheCleaner {
    fn id(&self) -> &'static str {
        "packages.brew.cache"
    }
    fn category(&self) -> &'static str {
        "Packages"
    }
    fn name(&self) -> &'static str {
        "Homebrew cache"
    }
    fn description(&self) -> &'static str {
        "Downloaded bottles and source tarballs kept by Homebrew. Use `brew cleanup` for the full cleanup; this removes the user-level cache only."
    }
    fn roots(&self, paths: &dyn PlatformPaths) -> Vec<PathBuf> {
        let Some(cache) = paths.user_cache_dir() else {
            return Vec::new();
        };
        let p = cache.join("Homebrew");
        if p.is_dir() {
            vec![p]
        } else {
            Vec::new()
        }
    }
}

pub struct YayCacheCleaner;
impl Cleaner for YayCacheCleaner {
    fn id(&self) -> &'static str {
        "packages.yay.cache"
    }
    fn category(&self) -> &'static str {
        "Packages"
    }
    fn name(&self) -> &'static str {
        "yay cache"
    }
    fn description(&self) -> &'static str {
        "AUR builds and downloads cached by yay (Arch Linux user-level helper)."
    }
    fn roots(&self, paths: &dyn PlatformPaths) -> Vec<PathBuf> {
        let Some(cache) = paths.user_cache_dir() else {
            return Vec::new();
        };
        let p = cache.join("yay");
        if p.is_dir() {
            vec![p]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    struct FixedPaths {
        home: Option<PathBuf>,
        cache: Option<PathBuf>,
        data: Option<PathBuf>,
    }
    impl PlatformPaths for FixedPaths {
        fn home_dir(&self) -> Option<PathBuf> {
            self.home.clone()
        }
        fn temp_dir(&self) -> PathBuf {
            std::env::temp_dir()
        }
        fn user_cache_dir(&self) -> Option<PathBuf> {
            self.cache.clone()
        }
        fn user_data_dir(&self) -> Option<PathBuf> {
            self.data.clone()
        }
        fn user_config_dir(&self) -> Option<PathBuf> {
            None
        }
        fn trash_dir(&self) -> Option<PathBuf> {
            None
        }
    }

    fn fresh(label: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("koscleaner-pkg-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn touch(p: &Path, bytes: usize) {
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        File::create(p)
            .unwrap()
            .write_all(&vec![0u8; bytes])
            .unwrap();
    }

    #[test]
    fn pip_detects_when_present() {
        let cache = fresh("pip");
        #[cfg(windows)]
        let pip = cache.join("pip").join("Cache");
        #[cfg(not(windows))]
        let pip = cache.join("pip");
        touch(&pip.join("wheels/x.whl"), 123);

        let paths = FixedPaths {
            home: None,
            cache: Some(cache.clone()),
            data: None,
        };
        let roots = PipCacheCleaner.roots(&paths);
        assert_eq!(roots.len(), 1);
        let report = crate::scanner::scan(&PipCacheCleaner, &paths);
        assert_eq!(report.total_size, 123);

        fs::remove_dir_all(&cache).ok();
    }

    #[test]
    fn npm_prefers_cacache_subdir() {
        let home = fresh("npm");
        touch(&home.join(".npm/_cacache/content-v2/x"), 50);

        let paths = FixedPaths {
            home: Some(home.clone()),
            cache: None,
            data: None,
        };
        let roots = NpmCacheCleaner.roots(&paths);
        // Sia _cacache che .npm esistono come dir e rispondono True.
        assert!(roots.iter().any(|r| r.ends_with("_cacache")));
        let report = crate::scanner::scan(&NpmCacheCleaner, &paths);
        assert_eq!(report.total_size, 50);

        fs::remove_dir_all(&home).ok();
    }

    #[test]
    fn homebrew_only_when_dir_exists() {
        let cache = fresh("brew");
        let paths = FixedPaths {
            home: None,
            cache: Some(cache.clone()),
            data: None,
        };
        assert!(HomebrewCacheCleaner.roots(&paths).is_empty());

        touch(&cache.join("Homebrew/downloads/x.tar"), 999);
        let roots = HomebrewCacheCleaner.roots(&paths);
        assert_eq!(roots.len(), 1);
        let report = crate::scanner::scan(&HomebrewCacheCleaner, &paths);
        assert_eq!(report.total_size, 999);

        fs::remove_dir_all(&cache).ok();
    }

    #[test]
    fn missing_user_cache_returns_empty() {
        let paths = FixedPaths {
            home: None,
            cache: None,
            data: None,
        };
        assert!(PipCacheCleaner.roots(&paths).is_empty());
        assert!(YayCacheCleaner.roots(&paths).is_empty());
        assert!(HomebrewCacheCleaner.roots(&paths).is_empty());
    }
}
