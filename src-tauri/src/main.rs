use std::path::PathBuf;
use tauri::Manager;

#[derive(serde::Serialize, serde::Deserialize)]
struct Settings {
    export_folder: String,
    export_format: String,
    #[serde(default)]
    template_folder: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            export_folder: String::new(),
            export_format: "csv".to_string(),
            template_folder: String::new(),
        }
    }
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join("settings.json"))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    let path = settings_path(&app)?;
    if !path.exists() {
        return Ok(Settings::default());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_settings(
    app: tauri::AppHandle,
    export_folder: String,
    export_format: String,
    template_folder: String,
) -> Result<(), String> {
    let path = settings_path(&app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let settings = Settings { export_folder, export_format, template_folder };
    let content = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Template {
    name: String,
    categories: Vec<(String, String)>,
}

#[tauri::command]
fn save_template(folder: String, name: String, categories: Vec<(String, String)>) -> Result<(), String> {
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let sanitized: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ')
        .collect::<String>()
        .replace(' ', "_");
    let path = std::path::Path::new(&folder).join(format!("{}.json", sanitized));
    let template = Template { name, categories };
    let content = serde_json::to_string_pretty(&template).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_templates(folder: String) -> Result<Vec<Template>, String> {
    let dir = std::path::Path::new(&folder);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut templates = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(t) = serde_json::from_str::<Template>(&content) {
                    templates.push(t);
                }
            }
        }
    }
    Ok(templates)
}

#[tauri::command]
async fn pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let folder = app.dialog().file().blocking_pick_folder();
    Ok(folder.map(|f| f.to_string()))
}

#[derive(serde::Deserialize)]
struct WorkEntry {
    hours: f64,
    categories: Vec<(String, String)>,
}

#[tauri::command]
fn export_workday(
    folder: String,
    format: String,
    date: String,
    entries: Vec<WorkEntry>,
) -> Result<(), String> {
    let dir = std::path::Path::new(&folder);
    if !dir.exists() {
        return Err(format!("Folder does not exist: {}", folder));
    }
    let ext = if format == "xlsx" { "xlsx" } else { "csv" };
    let path = dir.join(format!("workday_{}.{}", date, ext));
    if format == "xlsx" {
        export_xlsx(&path, &entries, &date)
    } else {
        export_csv(&path, &entries, &date)
    }
}

fn collect_columns(entries: &[WorkEntry]) -> Vec<String> {
    let mut cols: Vec<String> = Vec::new();
    for e in entries {
        for (k, _) in &e.categories {
            if !cols.contains(k) {
                cols.push(k.clone());
            }
        }
    }
    cols
}

fn export_csv(path: &std::path::Path, entries: &[WorkEntry], date: &str) -> Result<(), String> {
    let cols = collect_columns(entries);
    let mut wtr = csv::Writer::from_path(path).map_err(|e| e.to_string())?;
    let mut header = vec!["Date".to_string()];
    header.extend_from_slice(&cols);
    header.push("Hours".to_string());
    wtr.write_record(&header).map_err(|e| e.to_string())?;
    for e in entries {
        let map: std::collections::HashMap<&str, &str> =
            e.categories.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        let mut row = vec![date.to_string()];
        for col in &cols {
            row.push(map.get(col.as_str()).unwrap_or(&"").to_string());
        }
        row.push(format!("{:.1}", e.hours));
        wtr.write_record(&row).map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())
}

fn export_xlsx(path: &std::path::Path, entries: &[WorkEntry], date: &str) -> Result<(), String> {
    use rust_xlsxwriter::{Format, Workbook};
    let cols = collect_columns(entries);
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    let bold = Format::new().set_bold();
    ws.write_with_format(0, 0, "Date", &bold).map_err(|e| e.to_string())?;
    for (i, col) in cols.iter().enumerate() {
        ws.write_with_format(0, (i + 1) as u16, col.as_str(), &bold)
            .map_err(|e| e.to_string())?;
    }
    ws.write_with_format(0, (cols.len() + 1) as u16, "Hours", &bold).map_err(|e| e.to_string())?;
    for (ri, e) in entries.iter().enumerate() {
        let row = (ri + 1) as u32;
        ws.write(row, 0, date).map_err(|e| e.to_string())?;
        let map: std::collections::HashMap<&str, &str> =
            e.categories.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        for (ci, col) in cols.iter().enumerate() {
            ws.write(row, (ci + 1) as u16, *map.get(col.as_str()).unwrap_or(&""))
                .map_err(|e| e.to_string())?;
        }
        ws.write(row, (cols.len() + 1) as u16, e.hours).map_err(|e| e.to_string())?;
    }
    wb.save(path).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            pick_folder,
            export_workday,
            save_template,
            list_templates
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
