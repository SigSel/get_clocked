use std::path::PathBuf;

use get_clocked_shared::CategoryDefinition;
use tauri::Manager;

const MAX_CATEGORIES: usize = 100;
const MAX_VALUES_PER_CATEGORY: usize = 1000;

fn categories_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join("category_definitions.json"))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_categories(app: tauri::AppHandle) -> Result<Vec<CategoryDefinition>, String> {
    let path = categories_path(&app)?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let data = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_categories(
    app: tauri::AppHandle,
    categories: Vec<CategoryDefinition>,
) -> Result<(), String> {
    let path = categories_path(&app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(&categories).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_categories(path: String) -> Result<Vec<CategoryDefinition>, String> {
    let p = std::path::Path::new(&path);
    match p
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("xlsx") => import_categories_xlsx(p),
        _ => import_categories_csv(p),
    }
}

fn validate_import(categories: &[CategoryDefinition]) -> Result<(), String> {
    if categories.len() > MAX_CATEGORIES {
        return Err(format!(
            "Too many categories: {} (max {})",
            categories.len(),
            MAX_CATEGORIES
        ));
    }
    for cat in categories {
        if cat.values.len() > MAX_VALUES_PER_CATEGORY {
            return Err(format!(
                "Category '{}' has too many values: {} (max {})",
                cat.name,
                cat.values.len(),
                MAX_VALUES_PER_CATEGORY
            ));
        }
    }
    Ok(())
}

fn import_categories_csv(path: &std::path::Path) -> Result<Vec<CategoryDefinition>, String> {
    let mut reader = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|h| h.to_string())
        .collect();
    let mut columns: Vec<Vec<String>> = vec![vec![]; headers.len()];
    for record in reader.records() {
        let record = record.map_err(|e| e.to_string())?;
        for (i, value) in record.iter().enumerate() {
            if i < columns.len() && !value.is_empty() {
                let v = value.to_string();
                if !columns[i].contains(&v) {
                    columns[i].push(v);
                }
            }
        }
    }
    let result: Vec<CategoryDefinition> = headers
        .into_iter()
        .zip(columns)
        .filter(|(name, _)| !name.is_empty())
        .map(|(name, values)| CategoryDefinition { name, values })
        .collect();
    validate_import(&result)?;
    Ok(result)
}

fn import_categories_xlsx(path: &std::path::Path) -> Result<Vec<CategoryDefinition>, String> {
    use calamine::{open_workbook, Data, Reader, Xlsx};
    let mut wb: Xlsx<_> =
        open_workbook(path).map_err(|e: calamine::XlsxError| e.to_string())?;
    let range = wb
        .worksheet_range_at(0)
        .ok_or("No worksheets in file")?
        .map_err(|e: calamine::XlsxError| e.to_string())?;
    let mut rows = range.rows();
    let headers: Vec<String> = rows
        .next()
        .unwrap_or(&[])
        .iter()
        .map(|c| match c {
            Data::String(s) => s.clone(),
            Data::Empty => String::new(),
            other => other.to_string(),
        })
        .collect();
    let mut columns: Vec<Vec<String>> = vec![vec![]; headers.len()];
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i >= columns.len() {
                continue;
            }
            let v = match cell {
                Data::String(s) => s.clone(),
                Data::Empty => String::new(),
                other => other.to_string(),
            };
            if !v.is_empty() && !columns[i].contains(&v) {
                columns[i].push(v);
            }
        }
    }
    let result: Vec<CategoryDefinition> = headers
        .into_iter()
        .zip(columns)
        .filter(|(name, _)| !name.is_empty())
        .map(|(name, values)| CategoryDefinition { name, values })
        .collect();
    validate_import(&result)?;
    Ok(result)
}
