pub mod audit;
pub mod cleaners;
pub mod executor;
pub mod platform;
pub mod safety;
pub mod scanner;

use std::path::PathBuf;

use cleaners::Cleaner;
use executor::{ExecutionReport, ExecutionRequest};
use scanner::ScanReport;

fn cleaner_by_id(id: &str) -> Result<Box<dyn Cleaner>, String> {
    match id {
        "system.temp" => Ok(Box::new(cleaners::system::SystemTempCleaner)),
        "browsers.firefox.cache" => Ok(Box::new(cleaners::browser::FirefoxCacheCleaner)),
        "browsers.chrome.cache" => Ok(Box::new(cleaners::chromium::ChromeCacheCleaner)),
        "browsers.chromium.cache" => Ok(Box::new(cleaners::chromium::ChromiumCacheCleaner)),
        "browsers.brave.cache" => Ok(Box::new(cleaners::chromium::BraveCacheCleaner)),
        "packages.pip.cache" => Ok(Box::new(cleaners::packages::PipCacheCleaner)),
        "packages.npm.cache" => Ok(Box::new(cleaners::packages::NpmCacheCleaner)),
        "packages.brew.cache" => Ok(Box::new(cleaners::packages::HomebrewCacheCleaner)),
        "packages.yay.cache" => Ok(Box::new(cleaners::packages::YayCacheCleaner)),
        "trash.user" => Ok(Box::new(cleaners::trash::TrashCleaner)),
        other => Err(format!("unknown cleaner: {other}")),
    }
}

fn all_cleaners() -> Vec<Box<dyn Cleaner>> {
    vec![
        Box::new(cleaners::system::SystemTempCleaner),
        Box::new(cleaners::browser::FirefoxCacheCleaner),
        Box::new(cleaners::chromium::ChromeCacheCleaner),
        Box::new(cleaners::chromium::ChromiumCacheCleaner),
        Box::new(cleaners::chromium::BraveCacheCleaner),
        Box::new(cleaners::packages::PipCacheCleaner),
        Box::new(cleaners::packages::NpmCacheCleaner),
        Box::new(cleaners::packages::HomebrewCacheCleaner),
        Box::new(cleaners::packages::YayCacheCleaner),
        Box::new(cleaners::trash::TrashCleaner),
    ]
}

#[tauri::command]
fn scan_cleaner(id: String) -> Result<ScanReport, String> {
    let paths = platform::current();
    let cleaner = cleaner_by_id(&id)?;
    Ok(scanner::scan(cleaner.as_ref(), &paths))
}

#[tauri::command]
fn execute_cleaner(id: String, paths: Vec<PathBuf>) -> Result<ExecutionReport, String> {
    let platform_paths = platform::current();
    let cleaner = cleaner_by_id(&id)?;
    let request = ExecutionRequest { paths };
    Ok(executor::execute(
        cleaner.as_ref(),
        &platform_paths,
        &request,
    ))
}

#[tauri::command]
fn list_cleaners() -> Vec<CleanerInfo> {
    all_cleaners()
        .iter()
        .map(|c| CleanerInfo {
            id: c.id().to_string(),
            category: c.category().to_string(),
            name: c.name().to_string(),
            description: c.description().to_string(),
        })
        .collect()
}

#[derive(serde::Serialize)]
struct CleanerInfo {
    id: String,
    category: String,
    name: String,
    description: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_cleaner,
            execute_cleaner,
            list_cleaners
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
