use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Settings {
    pub export_folder: String,
    pub export_format: String,
    #[serde(default)]
    pub template_folder: String,
    #[serde(default = "default_date_format")]
    pub date_format: String,
    #[serde(default)]
    pub padding_columns: u32,
}

fn default_date_format() -> String {
    "YYYY-MM-DD".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            export_folder: String::new(),
            export_format: "csv".to_string(),
            template_folder: String::new(),
            date_format: default_date_format(),
            padding_columns: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WorkEntry {
    pub hours: f64,
    pub categories: Vec<(String, String)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CategoryDefinition {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Template {
    pub name: String,
    pub categories: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExportFormat {
    Csv,
    Xlsx,
}

impl ExportFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "csv",
            ExportFormat::Xlsx => "xlsx",
        }
    }
}

impl FromStr for ExportFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "xlsx" => Ok(ExportFormat::Xlsx),
            _ => Ok(ExportFormat::Csv),
        }
    }
}

impl Default for ExportFormat {
    fn default() -> Self {
        ExportFormat::Csv
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DateFormat {
    YyyyMmDd,
    YyyyDotMmDotDd,
    DdMmYyyy,
    DdDotMmDotYyyy,
}

impl DateFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            DateFormat::YyyyMmDd => "YYYY-MM-DD",
            DateFormat::YyyyDotMmDotDd => "YYYY.MM.DD",
            DateFormat::DdMmYyyy => "DD-MM-YYYY",
            DateFormat::DdDotMmDotYyyy => "DD.MM.YYYY",
        }
    }

    pub fn format_date(&self, iso_date: &str) -> String {
        let parts: Vec<&str> = iso_date.split('-').collect();
        if parts.len() != 3 {
            return iso_date.to_string();
        }
        let (y, m, d) = (parts[0], parts[1], parts[2]);
        match self {
            DateFormat::DdDotMmDotYyyy => format!("{}.{}.{}", d, m, y),
            DateFormat::DdMmYyyy => format!("{}-{}-{}", d, m, y),
            DateFormat::YyyyDotMmDotDd => format!("{}.{}.{}", y, m, d),
            DateFormat::YyyyMmDd => format!("{}-{}-{}", y, m, d),
        }
    }
}

impl FromStr for DateFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "YYYY.MM.DD" => Ok(DateFormat::YyyyDotMmDotDd),
            "DD-MM-YYYY" => Ok(DateFormat::DdMmYyyy),
            "DD.MM.YYYY" => Ok(DateFormat::DdDotMmDotYyyy),
            _ => Ok(DateFormat::YyyyMmDd),
        }
    }
}

impl Default for DateFormat {
    fn default() -> Self {
        DateFormat::YyyyMmDd
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportArgs {
    pub folder: String,
    pub format: String,
    pub date: String,
    pub date_format: String,
    pub entries: Vec<WorkEntry>,
    pub padding_columns: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_format_yyyy_mm_dd() {
        let fmt = DateFormat::YyyyMmDd;
        assert_eq!(fmt.format_date("2026-03-08"), "2026-03-08");
    }

    #[test]
    fn date_format_dd_dot_mm_dot_yyyy() {
        let fmt = DateFormat::DdDotMmDotYyyy;
        assert_eq!(fmt.format_date("2026-03-08"), "08.03.2026");
    }

    #[test]
    fn date_format_dd_mm_yyyy() {
        let fmt = DateFormat::DdMmYyyy;
        assert_eq!(fmt.format_date("2026-03-08"), "08-03-2026");
    }

    #[test]
    fn date_format_yyyy_dot_mm_dot_dd() {
        let fmt = DateFormat::YyyyDotMmDotDd;
        assert_eq!(fmt.format_date("2026-03-08"), "2026.03.08");
    }

    #[test]
    fn date_format_invalid_date() {
        let fmt = DateFormat::DdDotMmDotYyyy;
        // "not-a-date" splits into 3 parts, so it gets reformatted
        assert_eq!(fmt.format_date("not-a-date"), "date.a.not");
        // Only 2 parts — returned as-is
        assert_eq!(fmt.format_date("2026-03"), "2026-03");
    }

    #[test]
    fn export_format_from_str_roundtrip() {
        assert_eq!("csv".parse::<ExportFormat>().unwrap(), ExportFormat::Csv);
        assert_eq!("xlsx".parse::<ExportFormat>().unwrap(), ExportFormat::Xlsx);
        assert_eq!("unknown".parse::<ExportFormat>().unwrap(), ExportFormat::Csv);
    }

    #[test]
    fn date_format_from_str_roundtrip() {
        let formats = [
            DateFormat::YyyyMmDd,
            DateFormat::YyyyDotMmDotDd,
            DateFormat::DdMmYyyy,
            DateFormat::DdDotMmDotYyyy,
        ];
        for fmt in &formats {
            let s = fmt.as_str();
            let parsed: DateFormat = s.parse().unwrap();
            assert_eq!(&parsed, fmt);
        }
    }

    #[test]
    fn date_format_from_str_unknown_defaults() {
        let parsed: DateFormat = "garbage".parse().unwrap();
        assert_eq!(parsed, DateFormat::YyyyMmDd);
    }

    #[test]
    fn settings_default() {
        let s = Settings::default();
        assert_eq!(s.export_format, "csv");
        assert_eq!(s.date_format, "YYYY-MM-DD");
        assert_eq!(s.padding_columns, 0);
    }
}
