// Cleaner di sistema. Per ora copre SOLO la cartella temp: è la più sicura,
// pensata per essere svuotata, e non richiede privilegi elevati.

use std::path::PathBuf;

use crate::platform::PlatformPaths;

use super::Cleaner;

pub struct SystemTempCleaner;

impl Cleaner for SystemTempCleaner {
    fn id(&self) -> &'static str {
        "system.temp"
    }
    fn category(&self) -> &'static str {
        "System"
    }
    fn name(&self) -> &'static str {
        "Temporary files"
    }
    fn description(&self) -> &'static str {
        "Files left in the system temporary directory by applications and the OS."
    }
    fn roots(&self, paths: &dyn PlatformPaths) -> Vec<PathBuf> {
        vec![paths.temp_dir()]
    }
}
