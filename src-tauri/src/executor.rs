// L'executor è l'unico punto in cui KosCleaner cancella file. Tutto ciò che entra
// qui DEVE essere stato già scansionato e ri-validato: l'IPC tra frontend e
// backend è un trust boundary, quindi non ci si fida del payload del client.
// La sequenza per ogni path è:
//   1. safety::is_path_safe_to_delete (blocklist + home + traversal)
//   2. ensure_within_root contro le root dichiarate dal cleaner
//      (saltato sui symlink: rimuovere il link, non il target)
//   3. lstat per la size pre-cancellazione
//   4. fs::remove_file
// Al termine viene scritto un record nel log di audit.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::audit::{self, AuditFailure, AuditRecord};
use crate::cleaners::Cleaner;
use crate::platform::PlatformPaths;
use crate::safety;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionFailure {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionReport {
    pub cleaner_id: String,
    pub attempted: usize,
    pub deleted: usize,
    pub freed_bytes: u64,
    pub failures: Vec<ExecutionFailure>,
    pub audit_log_path: Option<PathBuf>,
}

pub fn execute(
    cleaner: &dyn Cleaner,
    paths_provider: &dyn PlatformPaths,
    request: &ExecutionRequest,
) -> ExecutionReport {
    let allowed_roots: Vec<PathBuf> = cleaner
        .roots(paths_provider)
        .into_iter()
        .filter_map(|r| r.canonicalize().ok())
        .collect();

    let mut report = ExecutionReport {
        cleaner_id: cleaner.id().to_string(),
        attempted: request.paths.len(),
        deleted: 0,
        freed_bytes: 0,
        failures: Vec::new(),
        audit_log_path: None,
    };

    for path in &request.paths {
        match try_delete(path, &allowed_roots) {
            Ok(size) => {
                report.deleted += 1;
                report.freed_bytes = report.freed_bytes.saturating_add(size);
            }
            Err(msg) => report.failures.push(ExecutionFailure {
                path: path.clone(),
                message: msg,
            }),
        }
    }

    let audit_record = AuditRecord {
        timestamp_unix: audit::now_unix(),
        cleaner_id: report.cleaner_id.clone(),
        attempted: report.attempted,
        deleted: report.deleted,
        freed_bytes: report.freed_bytes,
        failures: report
            .failures
            .iter()
            .map(|f| AuditFailure {
                path: f.path.clone(),
                message: f.message.clone(),
            })
            .collect(),
    };
    match audit::append(paths_provider, &audit_record) {
        Ok(p) => report.audit_log_path = Some(p),
        Err(e) => report.failures.push(ExecutionFailure {
            path: PathBuf::from("<audit-log>"),
            message: format!("failed to write audit log: {e}"),
        }),
    }

    report
}

fn try_delete(path: &Path, allowed_roots: &[PathBuf]) -> Result<u64, String> {
    safety::is_path_safe_to_delete(path).map_err(|e| format!("safety check failed: {e}"))?;

    let meta = fs::symlink_metadata(path).map_err(|e| format!("stat failed: {e}"))?;
    let file_type = meta.file_type();

    if !file_type.is_symlink() {
        if !allowed_roots
            .iter()
            .any(|root| safety::ensure_within_root(path, root).is_ok())
        {
            return Err("path is not within any of the cleaner's allowed roots".to_string());
        }
        if file_type.is_dir() {
            // L'executor lavora solo su file/symlink. Le directory restano: sono
            // strutture di sistema (es. /tmp stesso) o sotto-cartelle che possono
            // essere ripopolate. Cancellarle ricorsivamente è troppo rischioso
            // a questo livello.
            return Err("refusing to delete a directory".to_string());
        }
    }

    let size = meta.len();
    fs::remove_file(path).map_err(|e| format!("remove_file failed: {e}"))?;
    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    use crate::cleaners::Cleaner;

    struct FixedPaths {
        root: PathBuf,
        audit: PathBuf,
    }
    impl PlatformPaths for FixedPaths {
        fn home_dir(&self) -> Option<PathBuf> {
            None
        }
        fn temp_dir(&self) -> PathBuf {
            self.root.clone()
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
        fn audit_log_dir(&self) -> Option<PathBuf> {
            Some(self.audit.clone())
        }
    }

    struct FakeCleaner {
        root: PathBuf,
    }
    impl Cleaner for FakeCleaner {
        fn id(&self) -> &'static str {
            "test.fake"
        }
        fn category(&self) -> &'static str {
            "Test"
        }
        fn name(&self) -> &'static str {
            "Fake"
        }
        fn description(&self) -> &'static str {
            "Used in tests"
        }
        fn roots(&self, _: &dyn PlatformPaths) -> Vec<PathBuf> {
            vec![self.root.clone()]
        }
    }

    fn fresh_dirs(label: &str) -> (PathBuf, PathBuf) {
        let pid = std::process::id();
        let root = std::env::temp_dir().join(format!("koscleaner-exec-{label}-{pid}"));
        let audit = std::env::temp_dir().join(format!("koscleaner-execaudit-{label}-{pid}"));
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&audit);
        fs::create_dir_all(&root).unwrap();
        (root, audit)
    }

    fn write_file(path: &Path, bytes: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        File::create(path).unwrap().write_all(bytes).unwrap();
    }

    #[test]
    fn deletes_files_inside_root_and_writes_audit() {
        let (root, audit) = fresh_dirs("happy");
        let f1 = root.join("a.txt");
        let f2 = root.join("b.bin");
        write_file(&f1, &[0u8; 100]);
        write_file(&f2, &[0u8; 250]);

        let paths = FixedPaths {
            root: root.clone(),
            audit: audit.clone(),
        };
        let cleaner = FakeCleaner { root: root.clone() };
        let req = ExecutionRequest {
            paths: vec![f1.clone(), f2.clone()],
        };
        let report = execute(&cleaner, &paths, &req);

        assert_eq!(report.deleted, 2);
        assert_eq!(report.freed_bytes, 350);
        assert!(report.failures.is_empty(), "{:?}", report.failures);
        assert!(!f1.exists() && !f2.exists());
        assert!(report.audit_log_path.as_ref().map(|p| p.exists()).unwrap_or(false));

        let log_contents = fs::read_to_string(report.audit_log_path.unwrap()).unwrap();
        assert!(log_contents.contains("\"cleaner_id\":\"test.fake\""));
        assert!(log_contents.contains("\"deleted\":2"));

        fs::remove_dir_all(&root).ok();
        fs::remove_dir_all(&audit).ok();
    }

    #[test]
    fn rejects_path_outside_cleaner_roots() {
        let (root, audit) = fresh_dirs("outside");
        let other = std::env::temp_dir().join(format!(
            "koscleaner-exec-outside-other-{}",
            std::process::id()
        ));
        fs::create_dir_all(&other).unwrap();
        let stranger = other.join("stranger.txt");
        write_file(&stranger, &[0u8; 50]);

        let paths = FixedPaths {
            root: root.clone(),
            audit: audit.clone(),
        };
        let cleaner = FakeCleaner { root: root.clone() };
        let req = ExecutionRequest {
            paths: vec![stranger.clone()],
        };
        let report = execute(&cleaner, &paths, &req);

        assert_eq!(report.deleted, 0);
        assert_eq!(report.failures.len(), 1);
        assert!(report.failures[0].message.contains("not within"));
        assert!(stranger.exists(), "stranger file must NOT have been deleted");

        fs::remove_dir_all(&root).ok();
        fs::remove_dir_all(&other).ok();
        fs::remove_dir_all(&audit).ok();
    }

    #[test]
    fn refuses_to_delete_directories() {
        let (root, audit) = fresh_dirs("dir");
        let subdir = root.join("subdir");
        fs::create_dir_all(&subdir).unwrap();

        let paths = FixedPaths {
            root: root.clone(),
            audit: audit.clone(),
        };
        let cleaner = FakeCleaner { root: root.clone() };
        let req = ExecutionRequest {
            paths: vec![subdir.clone()],
        };
        let report = execute(&cleaner, &paths, &req);

        assert_eq!(report.deleted, 0);
        assert_eq!(report.failures.len(), 1);
        assert!(report.failures[0].message.contains("directory"));
        assert!(subdir.exists());

        fs::remove_dir_all(&root).ok();
        fs::remove_dir_all(&audit).ok();
    }

    #[cfg(unix)]
    #[test]
    fn deletes_symlink_without_following_target() {
        use std::os::unix::fs::symlink;
        let (root, audit) = fresh_dirs("symlink");
        let outside = std::env::temp_dir().join(format!(
            "koscleaner-exec-symlink-outside-{}",
            std::process::id()
        ));
        fs::create_dir_all(&outside).unwrap();
        let target = outside.join("target.txt");
        write_file(&target, &[0u8; 999]);

        let link = root.join("link");
        symlink(&target, &link).unwrap();

        let paths = FixedPaths {
            root: root.clone(),
            audit: audit.clone(),
        };
        let cleaner = FakeCleaner { root: root.clone() };
        let req = ExecutionRequest {
            paths: vec![link.clone()],
        };
        let report = execute(&cleaner, &paths, &req);

        assert_eq!(report.deleted, 1);
        assert!(report.failures.is_empty(), "{:?}", report.failures);
        assert!(!link.exists(), "symlink itself should be removed");
        assert!(target.exists(), "target outside root MUST NOT be deleted");

        fs::remove_dir_all(&root).ok();
        fs::remove_dir_all(&outside).ok();
        fs::remove_dir_all(&audit).ok();
    }

    #[test]
    fn audit_record_is_written_even_with_partial_failures() {
        let (root, audit) = fresh_dirs("partial");
        let real = root.join("real.txt");
        write_file(&real, &[0u8; 10]);
        let missing = root.join("missing.txt"); // non esiste

        let paths = FixedPaths {
            root: root.clone(),
            audit: audit.clone(),
        };
        let cleaner = FakeCleaner { root: root.clone() };
        let req = ExecutionRequest {
            paths: vec![real.clone(), missing.clone()],
        };
        let report = execute(&cleaner, &paths, &req);

        assert_eq!(report.deleted, 1);
        assert_eq!(report.failures.len(), 1);
        let log = report.audit_log_path.expect("audit must be written");
        let contents = fs::read_to_string(&log).unwrap();
        assert!(contents.contains("\"deleted\":1"));
        assert!(contents.contains("\"attempted\":2"));

        fs::remove_dir_all(&root).ok();
        fs::remove_dir_all(&audit).ok();
    }
}
