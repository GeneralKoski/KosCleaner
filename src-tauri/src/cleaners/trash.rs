// Cestino: tutto ciò che l'utente ha già "cancellato" via interfaccia. È la
// pulizia più sicura per definizione: l'utente ha già espresso il consenso
// quando ha mandato i file nel cestino.
//
// Layout per OS:
//   Linux (XDG):  $XDG_DATA_HOME/Trash/{files,info}/
//   macOS:        ~/.Trash/
//   Windows:      C:\$Recycle.Bin\<SID>\  (per-volume + SID; non gestito qui,
//                 lo Step 13 valuterà l'API Win32 IFileOperation o equivalente)

use std::path::PathBuf;

use crate::platform::PlatformPaths;

use super::Cleaner;

pub struct TrashCleaner;

impl Cleaner for TrashCleaner {
    fn id(&self) -> &'static str {
        "trash.user"
    }
    fn category(&self) -> &'static str {
        "Trash"
    }
    fn name(&self) -> &'static str {
        "Empty trash"
    }
    fn description(&self) -> &'static str {
        "Permanently delete every item already moved to the trash."
    }
    fn roots(&self, paths: &dyn PlatformPaths) -> Vec<PathBuf> {
        let Some(trash) = paths.trash_dir() else {
            return Vec::new();
        };
        if !trash.is_dir() {
            return Vec::new();
        }
        #[cfg(target_os = "linux")]
        {
            // XDG: i file reali sono in `files/`, i metadata in `info/`. Entrambi
            // sono safe da rimuovere; restano le directory parent.
            let mut out = Vec::new();
            for sub in &["files", "info"] {
                let p = trash.join(sub);
                if p.is_dir() {
                    out.push(p);
                }
            }
            out
        }
        #[cfg(not(target_os = "linux"))]
        {
            vec![trash]
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
        trash: Option<PathBuf>,
    }
    impl PlatformPaths for FixedPaths {
        fn home_dir(&self) -> Option<PathBuf> {
            None
        }
        fn temp_dir(&self) -> PathBuf {
            std::env::temp_dir()
        }
        fn user_cache_dir(&self) -> Option<PathBuf> {
            None
        }
        fn user_data_dir(&self) -> Option<PathBuf> {
            None
        }
        fn user_config_dir(&self) -> Option<PathBuf> {
            None
        }
        fn trash_dir(&self) -> Option<PathBuf> {
            self.trash.clone()
        }
    }

    fn fresh(label: &str) -> PathBuf {
        let p = std::env::temp_dir()
            .join(format!("koscleaner-trash-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn touch(p: &Path, bytes: usize) {
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        File::create(p).unwrap().write_all(&vec![0u8; bytes]).unwrap();
    }

    #[test]
    fn no_trash_dir_returns_empty() {
        let paths = FixedPaths { trash: None };
        assert!(TrashCleaner.roots(&paths).is_empty());
    }

    #[test]
    fn missing_trash_dir_returns_empty() {
        let p = std::env::temp_dir().join(format!(
            "koscleaner-trash-missing-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&p);
        let paths = FixedPaths { trash: Some(p) };
        assert!(TrashCleaner.roots(&paths).is_empty());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_returns_files_and_info_subdirs() {
        let trash = fresh("linux");
        touch(&trash.join("files/old.txt"), 100);
        touch(&trash.join("info/old.txt.trashinfo"), 50);

        let paths = FixedPaths {
            trash: Some(trash.clone()),
        };
        let roots = TrashCleaner.roots(&paths);
        assert_eq!(roots.len(), 2);
        let report = crate::scanner::scan(&TrashCleaner, &paths);
        assert_eq!(report.total_size, 150);

        fs::remove_dir_all(&trash).ok();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_returns_trash_dir_directly() {
        let trash = fresh("macos");
        touch(&trash.join("file1.txt"), 200);
        touch(&trash.join("subdir/inner.bin"), 300);

        let paths = FixedPaths {
            trash: Some(trash.clone()),
        };
        let roots = TrashCleaner.roots(&paths);
        assert_eq!(roots, vec![trash.clone()]);
        let report = crate::scanner::scan(&TrashCleaner, &paths);
        assert_eq!(report.total_size, 500);

        fs::remove_dir_all(&trash).ok();
    }
}
