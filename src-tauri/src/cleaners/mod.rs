// I cleaner sono moduli che dichiarano cosa scansionare. Non toccano mai il
// filesystem direttamente: espongono `roots()` che il scanner percorre, e in
// futuro l'executor cancellerà solo dopo conferma esplicita.

use std::path::PathBuf;

use crate::platform::PlatformPaths;

pub mod browser;
pub mod chromium;
pub mod packages;
pub mod system;
pub mod trash;

pub trait Cleaner {
    fn id(&self) -> &'static str;
    fn category(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn roots(&self, paths: &dyn PlatformPaths) -> Vec<PathBuf>;
}
