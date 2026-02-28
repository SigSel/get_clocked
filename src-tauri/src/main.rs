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

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct CategoryDefinition {
    name: String,
    values: Vec<String>,
}

fn categories_path(app: &tauri::AppHandle) -> PathBuf {
    app.path().app_config_dir().unwrap().join("category_definitions.json")
}

#[tauri::command]
fn get_categories(app: tauri::AppHandle) -> Vec<CategoryDefinition> {
    let path = categories_path(&app);
    if !path.exists() { return vec![]; }
    let data = std::fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&data).unwrap_or_default()
}

#[tauri::command]
fn save_categories(app: tauri::AppHandle, categories: Vec<CategoryDefinition>) -> Result<(), String> {
    let path = categories_path(&app);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(&categories).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
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

#[tauri::command]
fn export_monthly(folder: String, format: String, date: String, entries: Vec<WorkEntry>) -> Result<(), String> {
    let folder_path = std::path::Path::new(&folder);
    if !folder_path.exists() {
        return Err(format!("Export folder does not exist: {}", folder));
    }
    let month = &date[..7]; // "YYYY-MM" from "YYYY-MM-DD"
    match format.as_str() {
        "xlsx" => {
            let path = folder_path.join(format!("monthly_{}.xlsx", month));
            export_monthly_xlsx(&path, &entries, &date)
        }
        _ => {
            let path = folder_path.join(format!("monthly_{}.csv", month));
            export_monthly_csv(&path, &entries, &date)
        }
    }
}

fn export_monthly_csv(path: &std::path::Path, entries: &[WorkEntry], date: &str) -> Result<(), String> {
    if path.exists() {
        let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
        let raw_headers = rdr.headers().map_err(|e| e.to_string())?;
        let cols: Vec<String> = raw_headers.iter()
            .skip(1)
            .filter(|h| *h != "Hours")
            .map(|h| h.to_string())
            .collect();
        drop(rdr);

        let file = std::fs::OpenOptions::new()
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        let mut wtr = csv::Writer::from_writer(file);
        write_monthly_rows(&mut wtr, entries, &cols, date)?;
        wtr.flush().map_err(|e| e.to_string())
    } else {
        let cols = collect_columns(entries);
        let mut wtr = csv::Writer::from_path(path).map_err(|e| e.to_string())?;
        let mut header = vec!["Date".to_string()];
        header.extend_from_slice(&cols);
        header.push("Hours".to_string());
        wtr.write_record(&header).map_err(|e| e.to_string())?;
        write_monthly_rows(&mut wtr, entries, &cols, date)?;
        wtr.flush().map_err(|e| e.to_string())
    }
}

fn write_monthly_rows<W: std::io::Write>(
    wtr: &mut csv::Writer<W>,
    entries: &[WorkEntry],
    cols: &[String],
    date: &str,
) -> Result<(), String> {
    for e in entries {
        let map: std::collections::HashMap<&str, &str> =
            e.categories.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        let mut row = vec![date.to_string()];
        for col in cols {
            row.push(map.get(col.as_str()).unwrap_or(&"").to_string());
        }
        row.push(format!("{:.1}", e.hours));
        wtr.write_record(&row).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn export_monthly_xlsx(path: &std::path::Path, entries: &[WorkEntry], date: &str) -> Result<(), String> {
    use rust_xlsxwriter::{Format, Workbook};

    let (merged_cols, existing_rows): (Vec<String>, Vec<Vec<String>>) = if path.exists() {
        use calamine::{open_workbook, Data, Reader, Xlsx};
        let mut wb: Xlsx<_> = open_workbook(path).map_err(|e: calamine::XlsxError| e.to_string())?;
        let range = wb.worksheet_range_at(0)
            .ok_or("No worksheets in existing monthly file")?
            .map_err(|e: calamine::XlsxError| e.to_string())?;

        let all_rows: Vec<Vec<String>> = range.rows().map(|row: &[Data]| {
            row.iter().map(|cell| match cell {
                Data::String(s) => s.clone(),
                Data::Float(f) => format!("{:.1}", f),
                Data::Int(i) => i.to_string(),
                _ => String::new(),
            }).collect()
        }).collect();

        let existing_cols: Vec<String> = all_rows.get(0)
            .map(|h: &Vec<String>| h.iter().skip(1).filter(|c: &&String| c.as_str() != "Hours").cloned().collect())
            .unwrap_or_default();

        let new_cols = collect_columns(entries);
        let mut merged = existing_cols;
        for c in new_cols {
            if !merged.contains(&c) { merged.push(c); }
        }

        let data_rows: Vec<Vec<String>> = all_rows.into_iter().skip(1).collect();
        (merged, data_rows)
    } else {
        (collect_columns(entries), vec![])
    };

    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    let bold = Format::new().set_bold();

    ws.write_with_format(0, 0, "Date", &bold).map_err(|e| e.to_string())?;
    for (i, col) in merged_cols.iter().enumerate() {
        ws.write_with_format(0, (i + 1) as u16, col.as_str(), &bold).map_err(|e| e.to_string())?;
    }
    ws.write_with_format(0, (merged_cols.len() + 1) as u16, "Hours", &bold).map_err(|e| e.to_string())?;

    let existing_col_count = merged_cols.len() + 2;
    for (ri, row) in existing_rows.iter().enumerate() {
        let excel_row = (ri + 1) as u32;
        for (ci, val) in row.iter().enumerate().take(existing_col_count) {
            ws.write(excel_row, ci as u16, val.as_str()).map_err(|e| e.to_string())?;
        }
    }

    let row_offset = existing_rows.len() + 1;
    for (ri, e) in entries.iter().enumerate() {
        let excel_row = (row_offset + ri) as u32;
        ws.write(excel_row, 0, date).map_err(|e| e.to_string())?;
        let map: std::collections::HashMap<&str, &str> =
            e.categories.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        for (ci, col) in merged_cols.iter().enumerate() {
            ws.write(excel_row, (ci + 1) as u16, *map.get(col.as_str()).unwrap_or(&""))
                .map_err(|e| e.to_string())?;
        }
        ws.write(excel_row, (merged_cols.len() + 1) as u16, e.hours).map_err(|e| e.to_string())?;
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
            export_monthly,
            save_template,
            list_templates,
            get_categories,
            save_categories
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
