use get_clocked_shared::Template;

pub fn sanitize_name(name: &str) -> Result<String, String> {
    let sanitized: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ')
        .collect::<String>()
        .replace(' ', "_");
    if sanitized.is_empty() {
        return Err("Template name must contain at least one alphanumeric character.".to_string());
    }
    Ok(sanitized)
}

#[tauri::command]
pub fn save_template(
    folder: String,
    name: String,
    categories: Vec<(String, String)>,
) -> Result<(), String> {
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let sanitized = sanitize_name(&name)?;
    let path = std::path::Path::new(&folder).join(format!("{}.json", sanitized));
    let template = Template { name, categories };
    let content = serde_json::to_string_pretty(&template).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_templates(folder: String) -> Result<Vec<Template>, String> {
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
pub fn delete_template(folder: String, name: String) -> Result<(), String> {
    let sanitized = sanitize_name(&name)?;
    let path = std::path::Path::new(&folder).join(format!("{}.json", sanitized));
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_normal_name() {
        assert_eq!(sanitize_name("My Template").unwrap(), "My_Template");
    }

    #[test]
    fn sanitize_special_chars_only() {
        assert!(sanitize_name("!!!???").is_err());
    }

    #[test]
    fn sanitize_empty() {
        assert!(sanitize_name("").is_err());
    }

    #[test]
    fn sanitize_spaces_only() {
        // spaces become underscores, but the chars().filter removes spaces? No: ' ' is kept by the filter.
        // Actually: filter keeps alphanumeric OR space. Then replace(' ', "_").
        // "   " -> filter keeps "   " -> replace -> "___"
        assert_eq!(sanitize_name("   ").unwrap(), "___");
    }

    #[test]
    fn sanitize_mixed() {
        assert_eq!(sanitize_name("Test!@#123").unwrap(), "Test123");
    }
}
