// Modulo di sicurezza: ogni path che entra nell'executor DEVE passare di qui.
// Niente filesystem mutations senza il sigillo di approvazione di queste funzioni.

use std::fmt;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum SafetyError {
    Empty,
    NotAbsolute(PathBuf),
    ParentTraversal(PathBuf),
    CriticalSystemPath(PathBuf),
    HomeDirectory(PathBuf),
    OutsideRoot { path: PathBuf, root: PathBuf },
    CanonicalizeFailed { path: PathBuf, source: String },
}

impl fmt::Display for SafetyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SafetyError::Empty => write!(f, "path is empty"),
            SafetyError::NotAbsolute(p) => write!(f, "path is not absolute: {}", p.display()),
            SafetyError::ParentTraversal(p) => {
                write!(f, "path contains parent traversal (..): {}", p.display())
            }
            SafetyError::CriticalSystemPath(p) => {
                write!(f, "path is a critical system directory: {}", p.display())
            }
            SafetyError::HomeDirectory(p) => {
                write!(f, "path is the user home directory itself: {}", p.display())
            }
            SafetyError::OutsideRoot { path, root } => write!(
                f,
                "path {} is outside expected root {}",
                path.display(),
                root.display()
            ),
            SafetyError::CanonicalizeFailed { path, source } => {
                write!(f, "failed to canonicalize {}: {}", path.display(), source)
            }
        }
    }
}

impl std::error::Error for SafetyError {}

/// Verifica che il path sia ragionevolmente sicuro da cancellare.
/// NON garantisce che il file esista o che il chiamante abbia i permessi:
/// si limita a bloccare path notoriamente catastrofici (root, /etc, $HOME, ...).
pub fn is_path_safe_to_delete(path: &Path) -> Result<(), SafetyError> {
    if path.as_os_str().is_empty() {
        return Err(SafetyError::Empty);
    }
    if !path.is_absolute() {
        return Err(SafetyError::NotAbsolute(path.to_path_buf()));
    }
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(SafetyError::ParentTraversal(path.to_path_buf()));
    }

    let lexical = lexical_clean(path);
    let resolved = resolve_or_normalize(path)?;

    if let Some(home) = home_dir() {
        let home_resolved = resolve_or_normalize(&home).unwrap_or(home.clone());
        if resolved == home_resolved || lexical == home {
            return Err(SafetyError::HomeDirectory(resolved));
        }
    }

    for blocked in critical_paths() {
        let blocked_resolved = resolve_or_normalize(&blocked).unwrap_or_else(|_| blocked.clone());
        if resolved == blocked_resolved || lexical == blocked {
            return Err(SafetyError::CriticalSystemPath(resolved));
        }
    }

    Ok(())
}

// Pulizia lessicale: rimuove componenti CurDir/RootDir ridondanti senza toccare il filesystem.
fn lexical_clean(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Garantisce che `path` (dopo aver risolto eventuali symlink) sia interno a `root`.
/// Serve per i cleaner che operano in una directory specifica e devono assicurarsi
/// che un symlink al loro interno non esca verso un'area di sistema.
pub fn ensure_within_root(path: &Path, root: &Path) -> Result<(), SafetyError> {
    let resolved_path = resolve_or_normalize(path)?;
    let resolved_root = resolve_or_normalize(root)?;
    if resolved_path.starts_with(&resolved_root) {
        Ok(())
    } else {
        Err(SafetyError::OutsideRoot {
            path: resolved_path,
            root: resolved_root,
        })
    }
}

// Se il path esiste, canonicalize() (segue i symlink). Altrimenti normalizzazione
// lessicale del prefisso esistente + componenti residui.
fn resolve_or_normalize(path: &Path) -> Result<PathBuf, SafetyError> {
    if path.exists() {
        return path.canonicalize().map(strip_unc_prefix).map_err(|e| {
            SafetyError::CanonicalizeFailed {
                path: path.to_path_buf(),
                source: e.to_string(),
            }
        });
    }

    let mut existing = path.to_path_buf();
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    while !existing.exists() {
        match existing.file_name() {
            Some(name) => tail.push(name.to_os_string()),
            None => break,
        }
        if !existing.pop() {
            break;
        }
    }

    let base = if existing.as_os_str().is_empty() {
        path.to_path_buf()
    } else {
        existing.canonicalize().map(strip_unc_prefix).map_err(|e| {
            SafetyError::CanonicalizeFailed {
                path: existing.clone(),
                source: e.to_string(),
            }
        })?
    };

    let mut out = base;
    for name in tail.into_iter().rev() {
        out.push(name);
    }
    Ok(out)
}

#[cfg(windows)]
fn strip_unc_prefix(p: PathBuf) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        p
    }
}

#[cfg(not(windows))]
fn strip_unc_prefix(p: PathBuf) -> PathBuf {
    p
}

fn home_dir() -> Option<PathBuf> {
    #[allow(deprecated)]
    std::env::home_dir()
}

#[cfg(target_os = "linux")]
fn critical_paths() -> Vec<PathBuf> {
    [
        "/",
        "/bin",
        "/boot",
        "/dev",
        "/etc",
        "/home",
        "/lib",
        "/lib32",
        "/lib64",
        "/libx32",
        "/lost+found",
        "/media",
        "/mnt",
        "/opt",
        "/proc",
        "/root",
        "/run",
        "/sbin",
        "/srv",
        "/sys",
        "/tmp",
        "/usr",
        "/var",
    ]
    .iter()
    .map(PathBuf::from)
    .collect()
}

#[cfg(target_os = "macos")]
fn critical_paths() -> Vec<PathBuf> {
    [
        "/",
        "/Applications",
        "/Library",
        "/Network",
        "/System",
        "/Users",
        "/Volumes",
        "/bin",
        "/cores",
        "/dev",
        "/etc",
        "/home",
        "/opt",
        "/private",
        "/sbin",
        "/tmp",
        "/usr",
        "/var",
    ]
    .iter()
    .map(PathBuf::from)
    .collect()
}

#[cfg(target_os = "windows")]
fn critical_paths() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    for drive in b'A'..=b'Z' {
        out.push(PathBuf::from(format!("{}:\\", drive as char)));
    }
    if let Ok(systemroot) = std::env::var("SystemRoot") {
        out.push(PathBuf::from(systemroot));
    } else {
        out.push(PathBuf::from(r"C:\Windows"));
    }
    if let Ok(pf) = std::env::var("ProgramFiles") {
        out.push(PathBuf::from(pf));
    } else {
        out.push(PathBuf::from(r"C:\Program Files"));
    }
    if let Ok(pf86) = std::env::var("ProgramFiles(x86)") {
        out.push(PathBuf::from(pf86));
    } else {
        out.push(PathBuf::from(r"C:\Program Files (x86)"));
    }
    if let Ok(pd) = std::env::var("ProgramData") {
        out.push(PathBuf::from(pd));
    } else {
        out.push(PathBuf::from(r"C:\ProgramData"));
    }
    out.push(PathBuf::from(r"C:\Users"));
    out
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn critical_paths() -> Vec<PathBuf> {
    vec![PathBuf::from("/")]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn rejects_empty_path() {
        let err = is_path_safe_to_delete(Path::new("")).unwrap_err();
        assert_eq!(err, SafetyError::Empty);
    }

    #[test]
    fn rejects_relative_path() {
        let err = is_path_safe_to_delete(Path::new("foo/bar")).unwrap_err();
        assert!(matches!(err, SafetyError::NotAbsolute(_)));
    }

    #[test]
    fn rejects_parent_traversal() {
        #[cfg(unix)]
        let p = Path::new("/tmp/../etc");
        #[cfg(windows)]
        let p = Path::new(r"C:\Temp\..\Windows");
        let err = is_path_safe_to_delete(p).unwrap_err();
        assert!(matches!(err, SafetyError::ParentTraversal(_)));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_root() {
        let err = is_path_safe_to_delete(Path::new("/")).unwrap_err();
        assert!(matches!(err, SafetyError::CriticalSystemPath(_)));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_etc() {
        let err = is_path_safe_to_delete(Path::new("/etc")).unwrap_err();
        assert!(matches!(err, SafetyError::CriticalSystemPath(_)));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_usr() {
        let err = is_path_safe_to_delete(Path::new("/usr")).unwrap_err();
        assert!(matches!(err, SafetyError::CriticalSystemPath(_)));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_var() {
        let err = is_path_safe_to_delete(Path::new("/var")).unwrap_err();
        assert!(matches!(err, SafetyError::CriticalSystemPath(_)));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn rejects_home_root() {
        let err = is_path_safe_to_delete(Path::new("/home")).unwrap_err();
        assert!(matches!(err, SafetyError::CriticalSystemPath(_)));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn rejects_root_user() {
        let err = is_path_safe_to_delete(Path::new("/root")).unwrap_err();
        assert!(matches!(err, SafetyError::CriticalSystemPath(_)));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn rejects_system() {
        let err = is_path_safe_to_delete(Path::new("/System")).unwrap_err();
        assert!(matches!(err, SafetyError::CriticalSystemPath(_)));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn rejects_applications() {
        let err = is_path_safe_to_delete(Path::new("/Applications")).unwrap_err();
        assert!(matches!(err, SafetyError::CriticalSystemPath(_)));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn rejects_users_root() {
        let err = is_path_safe_to_delete(Path::new("/Users")).unwrap_err();
        assert!(matches!(err, SafetyError::CriticalSystemPath(_)));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn rejects_drive_root() {
        let err = is_path_safe_to_delete(Path::new(r"C:\")).unwrap_err();
        assert!(matches!(err, SafetyError::CriticalSystemPath(_)));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn rejects_windows_dir() {
        let err = is_path_safe_to_delete(Path::new(r"C:\Windows")).unwrap_err();
        assert!(matches!(err, SafetyError::CriticalSystemPath(_)));
    }

    #[test]
    fn rejects_user_home_directly() {
        let home = home_dir().expect("home dir must be resolvable in test env");
        let err = is_path_safe_to_delete(&home).unwrap_err();
        assert!(matches!(err, SafetyError::HomeDirectory(_)));
    }

    #[test]
    fn allows_subpath_inside_home() {
        let home = home_dir().expect("home dir must be resolvable in test env");
        let candidate = home.join(".cache").join("koscleaner-test-allow");
        // Non importa se il path esiste o no: la safety check non lo pretende.
        assert!(is_path_safe_to_delete(&candidate).is_ok());
    }

    #[test]
    fn allows_subpath_inside_temp() {
        let tmp = std::env::temp_dir().join("koscleaner-test-allow");
        assert!(is_path_safe_to_delete(&tmp).is_ok());
    }

    #[test]
    fn ensure_within_root_accepts_subpath() {
        let tmp = std::env::temp_dir();
        let inner = tmp.join("koscleaner-test-within");
        assert!(ensure_within_root(&inner, &tmp).is_ok());
    }

    #[test]
    fn ensure_within_root_rejects_sibling() {
        let tmp = std::env::temp_dir();
        let outside = tmp.parent().unwrap_or(Path::new("/")).to_path_buf();
        // outside non è dentro tmp
        let err = ensure_within_root(&outside, &tmp).unwrap_err();
        assert!(matches!(err, SafetyError::OutsideRoot { .. }));
    }

    #[cfg(unix)]
    #[test]
    fn ensure_within_root_detects_symlink_escape() {
        use std::os::unix::fs::symlink;
        let tmp = std::env::temp_dir();
        let root = tmp.join(format!("koscleaner-symtest-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let escape_link = root.join("escape");
        // crea symlink che punta fuori dalla root
        symlink("/etc", &escape_link).unwrap();

        let result = ensure_within_root(&escape_link, &root);
        let _ = fs::remove_dir_all(&root);
        assert!(matches!(result, Err(SafetyError::OutsideRoot { .. })));
    }
}
