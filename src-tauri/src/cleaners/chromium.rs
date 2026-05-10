// Chromium-family browsers (Chrome, Chromium, Brave): condividono lo stesso
// layout di profili (Default, Profile 1, Profile 2, ...) e gli stessi nomi di
// sotto-cartelle di cache (Cache, Code Cache, GPUCache, ecc.).

use std::fs;
use std::path::PathBuf;

use crate::platform::PlatformPaths;

use super::Cleaner;

const CACHE_SUBDIRS: &[&str] = &[
    "Cache",
    "Code Cache",
    "GPUCache",
    "DawnCache",
    "ShaderCache",
    "GrShaderCache",
];

// Brand identifica le directory del browser per i tre OS.
struct Brand {
    // path relativo a user_cache_dir (Linux) o None se la cache vive sotto user_data_dir.
    cache_dir: Option<&'static str>,
    // path relativo a user_data_dir o "User Data" sotto LOCALAPPDATA su Windows.
    profile_dir: &'static str,
}

#[cfg(target_os = "linux")]
const CHROME: Brand = Brand {
    cache_dir: Some("google-chrome"),
    profile_dir: "google-chrome",
};
#[cfg(target_os = "linux")]
const CHROMIUM: Brand = Brand {
    cache_dir: Some("chromium"),
    profile_dir: "chromium",
};
#[cfg(target_os = "linux")]
const BRAVE: Brand = Brand {
    cache_dir: Some("BraveSoftware/Brave-Browser"),
    profile_dir: "BraveSoftware/Brave-Browser",
};

#[cfg(target_os = "macos")]
const CHROME: Brand = Brand {
    cache_dir: Some("Google/Chrome"),
    profile_dir: "Google/Chrome",
};
#[cfg(target_os = "macos")]
const CHROMIUM: Brand = Brand {
    cache_dir: Some("Chromium"),
    profile_dir: "Chromium",
};
#[cfg(target_os = "macos")]
const BRAVE: Brand = Brand {
    cache_dir: Some("BraveSoftware/Brave-Browser"),
    profile_dir: "BraveSoftware/Brave-Browser",
};

#[cfg(target_os = "windows")]
const CHROME: Brand = Brand {
    cache_dir: None,
    profile_dir: r"Google\Chrome\User Data",
};
#[cfg(target_os = "windows")]
const CHROMIUM: Brand = Brand {
    cache_dir: None,
    profile_dir: r"Chromium\User Data",
};
#[cfg(target_os = "windows")]
const BRAVE: Brand = Brand {
    cache_dir: None,
    profile_dir: r"BraveSoftware\Brave-Browser\User Data",
};

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
const CHROME: Brand = Brand {
    cache_dir: None,
    profile_dir: "",
};
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
const CHROMIUM: Brand = Brand {
    cache_dir: None,
    profile_dir: "",
};
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
const BRAVE: Brand = Brand {
    cache_dir: None,
    profile_dir: "",
};

fn collect_roots(brand: &Brand, paths: &dyn PlatformPaths) -> Vec<PathBuf> {
    let mut bases: Vec<PathBuf> = Vec::new();
    if let (Some(sub), Some(cache)) = (brand.cache_dir, paths.user_cache_dir()) {
        bases.push(cache.join(sub));
    }
    if let Some(data) = paths.user_data_dir() {
        bases.push(data.join(brand.profile_dir));
    }

    let mut out = Vec::new();
    for base in bases {
        let entries = match fs::read_dir(&base) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let profile = entry.path();
            if !profile.is_dir() {
                continue;
            }
            // Soltanto profili: "Default" o "Profile N", "Guest Profile", ecc.
            let name = entry.file_name();
            let s = name.to_string_lossy();
            if !(s == "Default" || s.starts_with("Profile ") || s == "Guest Profile") {
                continue;
            }
            for sub in CACHE_SUBDIRS {
                let candidate = profile.join(sub);
                if candidate.is_dir() {
                    out.push(candidate);
                }
            }
            // Service Worker cache vive due livelli più sotto, sempre rigenerabile.
            let sw = profile.join("Service Worker").join("CacheStorage");
            if sw.is_dir() {
                out.push(sw);
            }
        }
    }
    out
}

pub struct ChromeCacheCleaner;
impl Cleaner for ChromeCacheCleaner {
    fn id(&self) -> &'static str {
        "browsers.chrome.cache"
    }
    fn category(&self) -> &'static str {
        "Browsers"
    }
    fn name(&self) -> &'static str {
        "Chrome cache"
    }
    fn description(&self) -> &'static str {
        "Disk cache, code cache and GPU/shader caches for every Chrome profile. Chrome will regenerate these on next launch."
    }
    fn roots(&self, paths: &dyn PlatformPaths) -> Vec<PathBuf> {
        collect_roots(&CHROME, paths)
    }
}

pub struct ChromiumCacheCleaner;
impl Cleaner for ChromiumCacheCleaner {
    fn id(&self) -> &'static str {
        "browsers.chromium.cache"
    }
    fn category(&self) -> &'static str {
        "Browsers"
    }
    fn name(&self) -> &'static str {
        "Chromium cache"
    }
    fn description(&self) -> &'static str {
        "Disk cache, code cache and GPU/shader caches for every Chromium profile."
    }
    fn roots(&self, paths: &dyn PlatformPaths) -> Vec<PathBuf> {
        collect_roots(&CHROMIUM, paths)
    }
}

pub struct BraveCacheCleaner;
impl Cleaner for BraveCacheCleaner {
    fn id(&self) -> &'static str {
        "browsers.brave.cache"
    }
    fn category(&self) -> &'static str {
        "Browsers"
    }
    fn name(&self) -> &'static str {
        "Brave cache"
    }
    fn description(&self) -> &'static str {
        "Disk cache, code cache and GPU/shader caches for every Brave profile."
    }
    fn roots(&self, paths: &dyn PlatformPaths) -> Vec<PathBuf> {
        collect_roots(&BRAVE, paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    struct FixedPaths {
        cache: PathBuf,
        data: PathBuf,
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
            Some(self.data.clone())
        }
        fn user_config_dir(&self) -> Option<PathBuf> {
            None
        }
        fn trash_dir(&self) -> Option<PathBuf> {
            None
        }
    }

    fn fresh(label: &str) -> (PathBuf, PathBuf) {
        let pid = std::process::id();
        let cache = std::env::temp_dir().join(format!("koscleaner-chr-{label}-c-{pid}"));
        let data = std::env::temp_dir().join(format!("koscleaner-chr-{label}-d-{pid}"));
        let _ = fs::remove_dir_all(&cache);
        let _ = fs::remove_dir_all(&data);
        (cache, data)
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
    fn no_install_returns_empty() {
        let (cache, data) = fresh("absent");
        let paths = FixedPaths {
            cache: cache.clone(),
            data: data.clone(),
        };
        assert!(ChromeCacheCleaner.roots(&paths).is_empty());
        assert!(BraveCacheCleaner.roots(&paths).is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn detects_chrome_default_and_profile_caches_macos() {
        let (cache, data) = fresh("chrome-mac");
        let chrome_cache_root = cache.join("Google/Chrome");
        touch(&chrome_cache_root.join("Default/Cache/data_0"), 1000);
        touch(&chrome_cache_root.join("Default/Code Cache/js/index"), 500);
        touch(&chrome_cache_root.join("Profile 1/Cache/data_0"), 200);
        touch(&chrome_cache_root.join("RandomFile.txt"), 10); // non profilo
                                                              // un service worker dentro user_data_dir
        let chrome_data_root = data.join("Google/Chrome");
        touch(
            &chrome_data_root.join("Default/Service Worker/CacheStorage/x"),
            333,
        );

        let paths = FixedPaths {
            cache: cache.clone(),
            data: data.clone(),
        };
        let roots = ChromeCacheCleaner.roots(&paths);

        // 3 cache dirs (Default/Cache, Default/Code Cache, Profile 1/Cache) + 1 SW.
        assert_eq!(roots.len(), 4, "got {roots:?}");
        let report = crate::scanner::scan(&ChromeCacheCleaner, &paths);
        assert_eq!(report.total_size, 1000 + 500 + 200 + 333);

        fs::remove_dir_all(&cache).ok();
        fs::remove_dir_all(&data).ok();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn brave_isolated_from_chrome_macos() {
        let (cache, data) = fresh("brave-iso");
        touch(
            &cache.join("BraveSoftware/Brave-Browser/Default/Cache/data"),
            777,
        );
        touch(&cache.join("Google/Chrome/Default/Cache/data"), 999);

        let paths = FixedPaths {
            cache: cache.clone(),
            data: data.clone(),
        };
        let brave = crate::scanner::scan(&BraveCacheCleaner, &paths);
        let chrome = crate::scanner::scan(&ChromeCacheCleaner, &paths);
        assert_eq!(brave.total_size, 777);
        assert_eq!(chrome.total_size, 999);

        fs::remove_dir_all(&cache).ok();
        fs::remove_dir_all(&data).ok();
    }
}
