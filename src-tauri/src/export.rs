use std::collections::HashMap;
use std::path::Path;

use get_clocked_shared::{DateFormat, WorkEntry};
use indexmap::IndexSet;
use tauri_plugin_dialog::DialogExt;

fn collect_columns(entries: &[WorkEntry]) -> Vec<String> {
    let mut set = IndexSet::new();
    for e in entries {
        for (k, _) in &e.categories {
            set.insert(k.clone());
        }
    }
    set.into_iter().collect()
}

fn build_header(cols: &[String], padding_columns: u32) -> Vec<String> {
    let mut header = vec!["Date".to_string()];
    header.extend_from_slice(cols);
    for _ in 0..padding_columns {
        header.push(String::new());
    }
    header.push("Hours".to_string());
    header
}

fn build_row(entry: &WorkEntry, cols: &[String], date: &str, padding_columns: u32) -> Vec<String> {
    let map: HashMap<&str, &str> = entry
        .categories
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let mut row = vec![date.to_string()];
    for col in cols {
        row.push(map.get(col.as_str()).unwrap_or(&"").to_string());
    }
    for _ in 0..padding_columns {
        row.push(String::new());
    }
    row.push(format!("{:.1}", entry.hours));
    row
}

#[tauri::command]
pub fn export_workday(
    app: tauri::AppHandle,
    folder: String,
    format: String,
    date: String,
    date_format: String,
    entries: Vec<WorkEntry>,
    padding_columns: u32,
) -> Result<(), String> {
    let dir = Path::new(&folder);
    if !dir.exists() {
        return Err(format!("Folder does not exist: {}", folder));
    }
    let ext = if format == "xlsx" { "xlsx" } else { "csv" };
    let path = dir.join(format!("workday_{}.{}", date, ext));

    // Overwrite confirmation
    if path.exists() {
        let confirmed = app
            .dialog()
            .message(format!(
                "File '{}' already exists. Overwrite?",
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file")
            ))
            .title("Confirm Overwrite")
            .blocking_show();
        if !confirmed {
            return Err("Export cancelled.".to_string());
        }
    }

    let date_fmt: DateFormat = date_format.parse().unwrap_or_default();
    let display_date = date_fmt.format_date(&date);
    if format == "xlsx" {
        export_xlsx(&path, &entries, &display_date, padding_columns)
    } else {
        export_csv(&path, &entries, &display_date, padding_columns)
    }
}

#[tauri::command]
pub fn export_monthly(
    folder: String,
    format: String,
    date: String,
    date_format: String,
    entries: Vec<WorkEntry>,
    padding_columns: u32,
) -> Result<(), String> {
    let folder_path = Path::new(&folder);
    if !folder_path.exists() {
        return Err(format!("Export folder does not exist: {}", folder));
    }
    let month = &date[..7]; // "YYYY-MM" from "YYYY-MM-DD"
    let date_fmt: DateFormat = date_format.parse().unwrap_or_default();
    let display_date = date_fmt.format_date(&date);
    match format.as_str() {
        "xlsx" => {
            let path = folder_path.join(format!("monthly_{}.xlsx", month));
            export_monthly_xlsx(&path, &entries, &display_date, padding_columns)
        }
        _ => {
            let path = folder_path.join(format!("monthly_{}.csv", month));
            export_monthly_csv(&path, &entries, &display_date, padding_columns)
        }
    }
}

fn export_csv(
    path: &Path,
    entries: &[WorkEntry],
    date: &str,
    padding_columns: u32,
) -> Result<(), String> {
    let cols = collect_columns(entries);
    let mut wtr = csv::Writer::from_path(path).map_err(|e| e.to_string())?;
    let header = build_header(&cols, padding_columns);
    wtr.write_record(&header).map_err(|e| e.to_string())?;
    for e in entries {
        let row = build_row(e, &cols, date, padding_columns);
        wtr.write_record(&row).map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())
}

fn export_xlsx(
    path: &Path,
    entries: &[WorkEntry],
    date: &str,
    padding_columns: u32,
) -> Result<(), String> {
    use rust_xlsxwriter::{Format, Workbook};
    let cols = collect_columns(entries);
    let header = build_header(&cols, padding_columns);
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    let bold = Format::new().set_bold();

    for (i, val) in header.iter().enumerate() {
        ws.write_with_format(0, i as u16, val.as_str(), &bold)
            .map_err(|e| e.to_string())?;
    }

    for (ri, e) in entries.iter().enumerate() {
        let row_data = build_row(e, &cols, date, padding_columns);
        let excel_row = (ri + 1) as u32;
        for (ci, val) in row_data.iter().enumerate() {
            // Write hours as a number
            if ci == row_data.len() - 1 {
                ws.write(excel_row, ci as u16, e.hours)
                    .map_err(|e| e.to_string())?;
            } else {
                ws.write(excel_row, ci as u16, val.as_str())
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    wb.save(path).map_err(|e| e.to_string())
}

fn export_monthly_csv(
    path: &Path,
    entries: &[WorkEntry],
    date: &str,
    padding_columns: u32,
) -> Result<(), String> {
    if path.exists() {
        let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
        let raw_headers = rdr.headers().map_err(|e| e.to_string())?;
        let cols: Vec<String> = raw_headers
            .iter()
            .skip(1)
            .filter(|h| *h != "Hours" && !h.is_empty())
            .map(|h| h.to_string())
            .collect();
        drop(rdr);

        let file = std::fs::OpenOptions::new()
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        let mut wtr = csv::Writer::from_writer(file);
        for e in entries {
            let row = build_row(e, &cols, date, padding_columns);
            wtr.write_record(&row).map_err(|e| e.to_string())?;
        }
        wtr.flush().map_err(|e| e.to_string())
    } else {
        let cols = collect_columns(entries);
        let mut wtr = csv::Writer::from_path(path).map_err(|e| e.to_string())?;
        let header = build_header(&cols, padding_columns);
        wtr.write_record(&header).map_err(|e| e.to_string())?;
        for e in entries {
            let row = build_row(e, &cols, date, padding_columns);
            wtr.write_record(&row).map_err(|e| e.to_string())?;
        }
        wtr.flush().map_err(|e| e.to_string())
    }
}

fn export_monthly_xlsx(
    path: &Path,
    entries: &[WorkEntry],
    date: &str,
    padding_columns: u32,
) -> Result<(), String> {
    use rust_xlsxwriter::{Format, Workbook};

    let (merged_cols, existing_rows): (Vec<String>, Vec<Vec<String>>) = if path.exists() {
        use calamine::{open_workbook, Data, Reader, Xlsx};
        let mut wb: Xlsx<_> =
            open_workbook(path).map_err(|e: calamine::XlsxError| e.to_string())?;
        let range = wb
            .worksheet_range_at(0)
            .ok_or("No worksheets in existing monthly file")?
            .map_err(|e: calamine::XlsxError| e.to_string())?;

        let all_rows: Vec<Vec<String>> = range
            .rows()
            .map(|row: &[Data]| {
                row.iter()
                    .map(|cell| match cell {
                        Data::String(s) => s.clone(),
                        Data::Float(f) => format!("{:.1}", f),
                        Data::Int(i) => i.to_string(),
                        _ => String::new(),
                    })
                    .collect()
            })
            .collect();

        let existing_cols: Vec<String> = all_rows
            .first()
            .map(|h: &Vec<String>| {
                h.iter()
                    .skip(1)
                    .filter(|c: &&String| c.as_str() != "Hours")
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        let new_cols = collect_columns(entries);
        let mut merged = existing_cols;
        for c in new_cols {
            if !merged.contains(&c) {
                merged.push(c);
            }
        }

        let data_rows: Vec<Vec<String>> = all_rows.into_iter().skip(1).collect();
        (merged, data_rows)
    } else {
        (collect_columns(entries), vec![])
    };

    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    let bold = Format::new().set_bold();
    let header = build_header(&merged_cols, padding_columns);

    for (i, val) in header.iter().enumerate() {
        ws.write_with_format(0, i as u16, val.as_str(), &bold)
            .map_err(|e| e.to_string())?;
    }

    let pad = padding_columns as usize;
    let existing_col_count = merged_cols.len() + 2 + pad;
    for (ri, row) in existing_rows.iter().enumerate() {
        let excel_row = (ri + 1) as u32;
        for (ci, val) in row.iter().enumerate().take(existing_col_count) {
            ws.write(excel_row, ci as u16, val.as_str())
                .map_err(|e| e.to_string())?;
        }
    }

    let row_offset = existing_rows.len() + 1;
    for (ri, e) in entries.iter().enumerate() {
        let row_data = build_row(e, &merged_cols, date, padding_columns);
        let excel_row = (row_offset + ri) as u32;
        for (ci, val) in row_data.iter().enumerate() {
            if ci == row_data.len() - 1 {
                ws.write(excel_row, ci as u16, e.hours)
                    .map_err(|e| e.to_string())?;
            } else {
                ws.write(excel_row, ci as u16, val.as_str())
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    wb.save(path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_columns_preserves_order_and_deduplicates() {
        let entries = vec![
            WorkEntry {
                hours: 1.0,
                categories: vec![
                    ("B".to_string(), "v1".to_string()),
                    ("A".to_string(), "v2".to_string()),
                ],
            },
            WorkEntry {
                hours: 2.0,
                categories: vec![
                    ("A".to_string(), "v3".to_string()),
                    ("C".to_string(), "v4".to_string()),
                ],
            },
        ];
        let cols = collect_columns(&entries);
        assert_eq!(cols, vec!["B", "A", "C"]);
    }

    #[test]
    fn build_header_with_padding() {
        let cols = vec!["Project".to_string(), "Task".to_string()];
        let header = build_header(&cols, 2);
        assert_eq!(header, vec!["Date", "Project", "Task", "", "", "Hours"]);
    }

    #[test]
    fn build_header_no_padding() {
        let cols = vec!["X".to_string()];
        let header = build_header(&cols, 0);
        assert_eq!(header, vec!["Date", "X", "Hours"]);
    }

    #[test]
    fn build_row_maps_categories() {
        let entry = WorkEntry {
            hours: 7.5,
            categories: vec![
                ("A".to_string(), "val_a".to_string()),
                ("C".to_string(), "val_c".to_string()),
            ],
        };
        let cols = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let row = build_row(&entry, &cols, "2026-03-08", 1);
        assert_eq!(row, vec!["2026-03-08", "val_a", "", "val_c", "", "7.5"]);
    }

    #[test]
    fn collect_columns_empty() {
        let cols = collect_columns(&[]);
        assert!(cols.is_empty());
    }
}
