// Audit log locale: ogni esecuzione è un record JSONL nel file `audit.log` sotto
// la audit dir della piattaforma. Niente rete, niente telemetria — solo un
// registro consultabile dall'utente.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::platform::PlatformPaths;

#[derive(Debug, Clone, Serialize)]
pub struct AuditRecord {
    pub timestamp_unix: u64,
    pub cleaner_id: String,
    pub attempted: usize,
    pub deleted: usize,
    pub freed_bytes: u64,
    pub failures: Vec<AuditFailure>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditFailure {
    pub path: PathBuf,
    pub message: String,
}

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn append(paths: &dyn PlatformPaths, record: &AuditRecord) -> io::Result<PathBuf> {
    let dir = paths.audit_log_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "audit log directory is not available on this platform",
        )
    })?;
    fs::create_dir_all(&dir)?;
    let log_path = dir.join("audit.log");
    let mut f = OpenOptions::new().create(true).append(true).open(&log_path)?;
    let line = serde_json::to_string(record)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    writeln!(f, "{line}")?;
    Ok(log_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct TempPaths {
        audit: PathBuf,
    }
    impl PlatformPaths for TempPaths {
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
            None
        }
        fn audit_log_dir(&self) -> Option<PathBuf> {
            Some(self.audit.clone())
        }
    }

    #[test]
    fn appends_record_creating_dir() {
        let dir = std::env::temp_dir()
            .join(format!("koscleaner-audit-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let paths = TempPaths { audit: dir.clone() };

        let rec = AuditRecord {
            timestamp_unix: 1_700_000_000,
            cleaner_id: "test.cleaner".into(),
            attempted: 3,
            deleted: 2,
            freed_bytes: 1024,
            failures: vec![AuditFailure {
                path: PathBuf::from("/tmp/x"),
                message: "permission denied".into(),
            }],
        };
        let log = append(&paths, &rec).unwrap();
        assert!(log.exists());

        let contents = fs::read_to_string(&log).unwrap();
        assert!(contents.contains("\"cleaner_id\":\"test.cleaner\""));
        assert!(contents.contains("\"deleted\":2"));
        assert!(contents.ends_with('\n'));

        // due esecuzioni → due righe.
        append(&paths, &rec).unwrap();
        let contents = fs::read_to_string(&log).unwrap();
        assert_eq!(contents.lines().count(), 2);

        fs::remove_dir_all(&dir).unwrap();
    }
}
