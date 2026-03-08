use std::path::PathBuf;

use get_clocked_shared::Settings;
use tauri::Manager;

pub fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path()
        .app_config_dir()
        .map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    let path = settings_path(&app)?;
    if !path.exists() {
        return Ok(Settings::default());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(
    app: tauri::AppHandle,
    export_folder: String,
    export_format: String,
    template_folder: String,
    date_format: String,
    padding_columns: u32,
) -> Result<(), String> {
    let path = settings_path(&app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let settings = Settings {
        export_folder,
        export_format,
        template_folder,
        date_format,
        padding_columns,
    };
    let content = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}
