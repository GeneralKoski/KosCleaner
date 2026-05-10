// Cleaner per i browser. Per ora copre la cache Firefox: una pulizia che il
// browser sa rigenerare al prossimo avvio, quindi a basso rischio.

use std::fs;
use std::path::PathBuf;

use crate::platform::PlatformPaths;

use super::Cleaner;

pub struct FirefoxCacheCleaner;

impl Cleaner for FirefoxCacheCleaner {
    fn id(&self) -> &'static str {
        "browsers.firefox.cache"
    }
    fn category(&self) -> &'static str {
        "Browsers"
    }
    fn name(&self) -> &'static str {
        "Firefox cache"
    }
    fn description(&self) -> &'static str {
        "Cached web content, startup cache and thumbnails for every Firefox profile. Firefox will regenerate these on next launch."
    }
    fn roots(&self, paths: &dyn PlatformPaths) -> Vec<PathBuf> {
        let Some(base) = firefox_cache_base(paths) else {
            return Vec::new();
        };
        let entries = match fs::read_dir(&base) {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };
        let subcaches = ["cache2", "startupCache", "thumbnails", "shader-cache"];
        let mut out = Vec::new();
        for entry in entries.flatten() {
            let profile = entry.path();
            if !profile.is_dir() {
                continue;
            }
            for sub in &subcaches {
                let candidate = profile.join(sub);
                if candidate.is_dir() {
                    out.push(candidate);
                }
            }
        }
        out
    }
}

#[cfg(target_os = "linux")]
fn firefox_cache_base(paths: &dyn PlatformPaths) -> Option<PathBuf> {
    paths.user_cache_dir().map(|c| c.join("mozilla/firefox"))
}

#[cfg(target_os = "macos")]
fn firefox_cache_base(paths: &dyn PlatformPaths) -> Option<PathBuf> {
    paths.user_cache_dir().map(|c| c.join("Firefox/Profiles"))
}

#[cfg(target_os = "windows")]
fn firefox_cache_base(paths: &dyn PlatformPaths) -> Option<PathBuf> {
    paths
        .user_cache_dir()
        .map(|c| c.join(r"Mozilla\Firefox\Profiles"))
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn firefox_cache_base(_: &dyn PlatformPaths) -> Option<PathBuf> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    struct FixedPaths {
        cache: PathBuf,
    }
    impl PlatformPaths for FixedPaths {
        fn home_dir(&self) -> Option<PathBuf> {
            None
        }
        fn temp_dir(&self) -> PathBuf {
            std::env::temp_dir()
        }
        fn user_cache_dir(&self) -> Option<PathBuf> {
            Some(self.cache.clone())
        }
        fn user_data_dir(&self) -> Option<PathBuf> {
            None
        }
        fn user_config_dir(&self) -> Option<PathBuf> {
            None
        }
        fn trash_dir(&self) -> Option<PathBuf> {
            None
        }
    }

    fn unique_cache(label: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "koscleaner-firefox-{}-{}",
            label,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn write_file(path: &std::path::Path, bytes: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        File::create(path).unwrap().write_all(bytes).unwrap();
    }

    #[cfg(target_os = "linux")]
    fn make_profile(cache_root: &std::path::Path, profile_id: &str, sub: &str) -> PathBuf {
        let p = cache_root
            .join("mozilla/firefox")
            .join(profile_id)
            .join(sub);
        fs::create_dir_all(&p).unwrap();
        p
    }
    #[cfg(target_os = "macos")]
    fn make_profile(cache_root: &std::path::Path, profile_id: &str, sub: &str) -> PathBuf {
        let p = cache_root
            .join("Firefox/Profiles")
            .join(profile_id)
            .join(sub);
        fs::create_dir_all(&p).unwrap();
        p
    }
    #[cfg(target_os = "windows")]
    fn make_profile(cache_root: &std::path::Path, profile_id: &str, sub: &str) -> PathBuf {
        let p = cache_root
            .join(r"Mozilla\Firefox\Profiles")
            .join(profile_id)
            .join(sub);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn no_firefox_install_yields_empty_roots() {
        let cache = unique_cache("absent");
        let paths = FixedPaths {
            cache: cache.clone(),
        };
        assert!(FirefoxCacheCleaner.roots(&paths).is_empty());
        fs::remove_dir_all(&cache).ok();
    }

    #[test]
    fn detects_subcaches_across_profiles() {
        let cache = unique_cache("multi");

        let p1c2 = make_profile(&cache, "abcd1234.default", "cache2");
        let p1sc = make_profile(&cache, "abcd1234.default", "startupCache");
        let p2c2 = make_profile(&cache, "9999zzzz.dev-edition", "cache2");
        // crea anche un file dentro per essere realistici
        write_file(&p1c2.join("entry-1"), &[0u8; 256]);
        write_file(&p1sc.join("scriptCache.bin"), &[0u8; 64]);
        write_file(&p2c2.join("entry-1"), &[0u8; 128]);

        let paths = FixedPaths {
            cache: cache.clone(),
        };
        let roots = FirefoxCacheCleaner.roots(&paths);

        assert!(roots.iter().any(|r| r == &p1c2));
        assert!(roots.iter().any(|r| r == &p1sc));
        assert!(roots.iter().any(|r| r == &p2c2));
        assert_eq!(roots.len(), 3);

        fs::remove_dir_all(&cache).ok();
    }

    #[test]
    fn skips_non_directory_entries() {
        let cache = unique_cache("nondir");

        // un profilo regolare
        let p1 = make_profile(&cache, "real.default", "cache2");
        write_file(&p1.join("e"), &[0u8; 10]);

        // un file sparso nella base dei profili: non deve essere considerato
        #[cfg(target_os = "linux")]
        let stray = cache.join("mozilla/firefox/strayfile.txt");
        #[cfg(target_os = "macos")]
        let stray = cache.join("Firefox/Profiles/strayfile.txt");
        #[cfg(target_os = "windows")]
        let stray = cache.join(r"Mozilla\Firefox\Profiles\strayfile.txt");
        write_file(&stray, &[0u8; 4]);

        let paths = FixedPaths {
            cache: cache.clone(),
        };
        let roots = FirefoxCacheCleaner.roots(&paths);
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0], p1);

        fs::remove_dir_all(&cache).ok();
    }

    #[test]
    fn integrates_with_scanner_and_reports_files() {
        let cache = unique_cache("scan");
        let p = make_profile(&cache, "scan.default", "cache2");
        write_file(&p.join("a"), &[0u8; 100]);
        write_file(&p.join("nested/b"), &[0u8; 400]);

        let paths = FixedPaths {
            cache: cache.clone(),
        };
        let report = crate::scanner::scan(&FirefoxCacheCleaner, &paths);
        assert_eq!(report.items.len(), 2);
        assert_eq!(report.total_size, 500);
        assert!(report.errors.is_empty(), "{:?}", report.errors);

        fs::remove_dir_all(&cache).ok();
    }
}
