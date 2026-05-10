pub mod cleaners;
pub mod platform;
pub mod safety;
pub mod scanner;

use cleaners::Cleaner;
use scanner::ScanReport;

#[tauri::command]
fn scan_cleaner(id: String) -> Result<ScanReport, String> {
    let paths = platform::current();
    let cleaner: Box<dyn Cleaner> = match id.as_str() {
        "system.temp" => Box::new(cleaners::system::SystemTempCleaner),
        other => return Err(format!("unknown cleaner: {other}")),
    };
    Ok(scanner::scan(cleaner.as_ref(), &paths))
}

#[tauri::command]
fn list_cleaners() -> Vec<CleanerInfo> {
    let cleaners: Vec<Box<dyn Cleaner>> = vec![Box::new(cleaners::system::SystemTempCleaner)];
    cleaners
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
        .invoke_handler(tauri::generate_handler![scan_cleaner, list_cleaners])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
