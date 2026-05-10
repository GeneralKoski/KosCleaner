// Scanner dry-run: percorre i root dichiarati dal cleaner e raccoglie file +
// dimensioni. NON modifica nulla. Ogni path passa per safety::is_path_safe_to_delete
// e ensure_within_root prima di finire nel report.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::cleaners::Cleaner;
use crate::platform::PlatformPaths;
use crate::safety;

#[derive(Debug, Clone, Serialize)]
pub struct ScanItem {
    pub path: PathBuf,
    pub size: u64,
    pub is_symlink: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanError {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanReport {
    pub cleaner_id: String,
    pub category: String,
    pub name: String,
    pub items: Vec<ScanItem>,
    pub total_size: u64,
    pub errors: Vec<ScanError>,
}

impl ScanReport {
    fn new(cleaner: &dyn Cleaner) -> Self {
        Self {
            cleaner_id: cleaner.id().to_string(),
            category: cleaner.category().to_string(),
            name: cleaner.name().to_string(),
            items: Vec::new(),
            total_size: 0,
            errors: Vec::new(),
        }
    }

    fn push_item(&mut self, path: PathBuf, size: u64, is_symlink: bool) {
        self.total_size = self.total_size.saturating_add(size);
        self.items.push(ScanItem {
            path,
            size,
            is_symlink,
        });
    }

    fn push_error(&mut self, path: PathBuf, message: impl Into<String>) {
        self.errors.push(ScanError {
            path,
            message: message.into(),
        });
    }
}

pub fn scan(cleaner: &dyn Cleaner, paths: &dyn PlatformPaths) -> ScanReport {
    let mut report = ScanReport::new(cleaner);
    for root in cleaner.roots(paths) {
        scan_root(&root, &mut report);
    }
    report
}

fn scan_root(root: &Path, report: &mut ScanReport) {
    let canonical_root = match root.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            // Root mancante non è un errore fatale: una temp dir vuota o non esistente
            // è semplicemente "niente da pulire". La logghiamo solo se non è NotFound.
            if e.kind() != std::io::ErrorKind::NotFound {
                report.push_error(root.to_path_buf(), format!("cannot access root: {e}"));
            }
            return;
        }
    };
    walk(&canonical_root, &canonical_root, report);
}

fn walk(root: &Path, dir: &Path, report: &mut ScanReport) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            report.push_error(dir.to_path_buf(), format!("read_dir failed: {e}"));
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // symlink_metadata non segue il symlink: ci interessa l'oggetto stesso.
        let meta = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                report.push_error(path.clone(), format!("stat failed: {e}"));
                continue;
            }
        };
        let file_type = meta.file_type();

        // Validazione safety: per i symlink basta is_path_safe_to_delete (che già
        // canonicalizza e blocca /tmp/x → /etc). ensure_within_root su un symlink
        // lo classificherebbe come "fuori dalla root" perché ne segue il target,
        // ma `remove_file` cancella il link, non il target — operazione sicura.
        // Sui file e directory reali invece pretendiamo che restino dentro la root,
        // così un symlink-trick a livello di antenato non riesce a esfiltrare.
        if let Err(e) = safety::is_path_safe_to_delete(&path) {
            report.push_error(path.clone(), format!("blocked by safety: {e}"));
            continue;
        }
        if !file_type.is_symlink() {
            if let Err(e) = safety::ensure_within_root(&path, root) {
                report.push_error(path.clone(), format!("escapes scan root: {e}"));
                continue;
            }
        }

        if file_type.is_symlink() {
            report.push_item(path, meta.len(), true);
        } else if file_type.is_dir() {
            walk(root, &path, report);
        } else {
            report.push_item(path, meta.len(), false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    use crate::platform::PlatformPaths;

    struct FixedPaths {
        temp: PathBuf,
    }
    impl PlatformPaths for FixedPaths {
        fn home_dir(&self) -> Option<PathBuf> {
            None
        }
        fn temp_dir(&self) -> PathBuf {
            self.temp.clone()
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
            None
        }
    }

    fn unique_root(label: &str) -> PathBuf {
        let p =
            std::env::temp_dir().join(format!("koscleaner-scan-{}-{}", label, std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn write_file(path: &Path, bytes: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = File::create(path).unwrap();
        f.write_all(bytes).unwrap();
    }

    #[test]
    fn scans_flat_temp_dir() {
        let root = unique_root("flat");
        write_file(&root.join("a.txt"), &[0u8; 100]);
        write_file(&root.join("b.bin"), &[0u8; 250]);

        let paths = FixedPaths { temp: root.clone() };
        let cleaner = crate::cleaners::system::SystemTempCleaner;
        let report = scan(&cleaner, &paths);

        assert_eq!(report.items.len(), 2);
        assert_eq!(report.total_size, 350);
        assert!(
            report.errors.is_empty(),
            "unexpected errors: {:?}",
            report.errors
        );

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn scans_nested_dirs_recursively() {
        let root = unique_root("nested");
        write_file(&root.join("top.txt"), &[0u8; 10]);
        write_file(&root.join("sub/inner.txt"), &[0u8; 20]);
        write_file(&root.join("sub/deeper/leaf.txt"), &[0u8; 30]);

        let paths = FixedPaths { temp: root.clone() };
        let cleaner = crate::cleaners::system::SystemTempCleaner;
        let report = scan(&cleaner, &paths);

        assert_eq!(report.items.len(), 3);
        assert_eq!(report.total_size, 60);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn empty_or_missing_root_yields_empty_report() {
        let root =
            std::env::temp_dir().join(format!("koscleaner-scan-missing-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);

        let paths = FixedPaths { temp: root.clone() };
        let cleaner = crate::cleaners::system::SystemTempCleaner;
        let report = scan(&cleaner, &paths);

        assert!(report.items.is_empty());
        assert_eq!(report.total_size, 0);
        assert!(
            report.errors.is_empty(),
            "unexpected errors: {:?}",
            report.errors
        );
    }

    #[cfg(unix)]
    #[test]
    fn does_not_follow_symlinks_into_other_dirs() {
        use std::os::unix::fs::symlink;
        let root = unique_root("symlink");
        let outside = unique_root("symlink-outside");
        write_file(&outside.join("secret.txt"), &[0u8; 9999]);

        // dentro la temp dir piazziamo un symlink che punta a una directory esterna.
        symlink(&outside, root.join("escape")).unwrap();
        write_file(&root.join("legit.txt"), &[0u8; 5]);

        let paths = FixedPaths { temp: root.clone() };
        let cleaner = crate::cleaners::system::SystemTempCleaner;
        let report = scan(&cleaner, &paths);

        // il symlink viene riportato come item (lo cancelleremo in sé), ma non si
        // entra nel target: nessun riferimento a outside/secret.txt nel report.
        assert!(report
            .items
            .iter()
            .any(|i| i.path.ends_with("legit.txt") && !i.is_symlink));
        assert!(report
            .items
            .iter()
            .any(|i| i.path.ends_with("escape") && i.is_symlink));
        assert!(!report
            .items
            .iter()
            .any(|i| i.path.to_string_lossy().contains("secret.txt")));

        fs::remove_dir_all(&root).unwrap();
        fs::remove_dir_all(&outside).unwrap();
    }
}
