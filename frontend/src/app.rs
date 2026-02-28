use std::sync::Arc;

use dominator::{clone, html, Dom};
use futures_signals::signal::Mutable;
use futures_signals::signal_vec::MutableVec;

use crate::pages;

#[derive(Clone, PartialEq)]
pub enum AppPage {
    Home,
    Settings,
    RegisterWorkday,
    TemplateMaker,
    CategoryManager,
}

#[derive(Clone, PartialEq)]
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

    pub fn from_str(s: &str) -> Self {
        match s {
            "xlsx" => ExportFormat::Xlsx,
            _ => ExportFormat::Csv,
        }
    }
}

pub struct DraftCategory {
    pub key: Mutable<String>,
    pub value: Mutable<String>,
}

impl DraftCategory {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            key: Mutable::new(String::new()),
            value: Mutable::new(String::new()),
        })
    }
}

pub struct DraftEntry {
    pub hours: Mutable<String>,
    pub categories: MutableVec<Arc<DraftCategory>>,
}

impl DraftEntry {
    pub fn new() -> Self {
        Self {
            hours: Mutable::new(String::new()),
            categories: MutableVec::new(),
        }
    }

    pub fn reset(&self) {
        self.hours.set(String::new());
        self.categories.lock_mut().clear();
    }
}

#[derive(Clone, serde::Serialize)]
pub struct WorkEntry {
    pub hours: f64,
    pub categories: Vec<(String, String)>,
}

#[derive(Clone, serde::Deserialize)]
pub struct TemplateData {
    pub name: String,
    pub categories: Vec<(String, String)>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CategoryDefinition {
    pub name: String,
    pub values: Vec<String>,
}

pub struct DraftCategoryDefinition {
    pub name: Mutable<String>,
    pub values: MutableVec<Arc<Mutable<String>>>,
}

impl DraftCategoryDefinition {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            name: Mutable::new(String::new()),
            values: MutableVec::new(),
        })
    }

    pub fn from_definition(def: &CategoryDefinition) -> Arc<Self> {
        let d = Arc::new(Self {
            name: Mutable::new(def.name.clone()),
            values: MutableVec::new(),
        });
        let mut lock = d.values.lock_mut();
        for v in &def.values {
            lock.push_cloned(Arc::new(Mutable::new(v.clone())));
        }
        drop(lock);
        d
    }
}

pub struct CategoryManagerState {
    pub definitions: MutableVec<Arc<DraftCategoryDefinition>>,
    pub status_msg: Mutable<Option<String>>,
    pub error_msg: Mutable<Option<String>>,
}

impl CategoryManagerState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            definitions: MutableVec::new(),
            status_msg: Mutable::new(None),
            error_msg: Mutable::new(None),
        })
    }
}

pub struct TemplateMakerState {
    pub name: Mutable<String>,
    pub categories: MutableVec<Arc<DraftCategory>>,
    pub error_msg: Mutable<Option<String>>,
    pub status_msg: Mutable<Option<String>>,
    pub templates: MutableVec<TemplateData>,
    pub editing_original_name: Mutable<Option<String>>,
}

impl TemplateMakerState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            name: Mutable::new(String::new()),
            categories: MutableVec::new(),
            error_msg: Mutable::new(None),
            status_msg: Mutable::new(None),
            templates: MutableVec::new(),
            editing_original_name: Mutable::new(None),
        })
    }

    pub fn reset(&self) {
        self.name.set(String::new());
        self.categories.lock_mut().clear();
        self.error_msg.set(None);
        self.status_msg.set(None);
        self.editing_original_name.set(None);
    }
}

pub struct WorkdayState {
    pub date: Mutable<String>,
    pub draft_visible: Mutable<bool>,
    pub draft: DraftEntry,
    pub entries: MutableVec<WorkEntry>,
    pub total_hours: Mutable<f64>,
    pub error_msg: Mutable<Option<String>>,
    pub status_msg: Mutable<Option<String>>,
    pub include_monthly: Mutable<bool>,
}

impl WorkdayState {
    pub fn new() -> Self {
        Self {
            date: Mutable::new(today_date_string()),
            draft_visible: Mutable::new(false),
            draft: DraftEntry::new(),
            entries: MutableVec::new(),
            total_hours: Mutable::new(0.0),
            error_msg: Mutable::new(None),
            status_msg: Mutable::new(None),
            include_monthly: Mutable::new(false),
        }
    }

    pub fn reset(&self) {
        self.date.set(today_date_string());
        self.draft_visible.set(false);
        self.draft.reset();
        self.entries.lock_mut().clear();
        self.total_hours.set(0.0);
        self.error_msg.set(None);
        self.status_msg.set(None);
        self.include_monthly.set(false);
    }
}

fn today_date_string() -> String {
    let d = js_sys::Date::new_0();
    format!("{:04}-{:02}-{:02}", d.get_full_year(), d.get_month() + 1, d.get_date())
}

pub struct AppState {
    pub page: Mutable<AppPage>,
    pub export_folder: Mutable<String>,
    pub export_format: Mutable<ExportFormat>,
    pub template_folder: Mutable<String>,
    pub workday: Arc<WorkdayState>,
    pub template_maker: Arc<TemplateMakerState>,
    pub category_definitions: Mutable<Vec<CategoryDefinition>>,
    pub category_manager: Arc<CategoryManagerState>,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Settings {
    export_folder: String,
    export_format: String,
    #[serde(default)]
    template_folder: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveArgs {
    export_folder: String,
    export_format: String,
    template_folder: String,
}

impl AppState {
    pub async fn load() -> Self {
        let settings: Settings = async {
            let js_val = tauri_wasm::invoke("get_settings")
                .await
                .map_err(|e| e.to_string())?;
            serde_wasm_bindgen::from_value::<Settings>(js_val).map_err(|e| e.to_string())
        }
        .await
        .unwrap_or_default();

        let categories: Vec<CategoryDefinition> = async {
            let js_val = tauri_wasm::invoke("get_categories").await.map_err(|e| e.to_string())?;
            serde_wasm_bindgen::from_value::<Vec<CategoryDefinition>>(js_val).map_err(|e| e.to_string())
        }
        .await
        .unwrap_or_default();

        AppState {
            page: Mutable::new(AppPage::Home),
            export_folder: Mutable::new(settings.export_folder),
            export_format: Mutable::new(ExportFormat::from_str(&settings.export_format)),
            template_folder: Mutable::new(settings.template_folder),
            workday: Arc::new(WorkdayState::new()),
            template_maker: TemplateMakerState::new(),
            category_definitions: Mutable::new(categories),
            category_manager: CategoryManagerState::new(),
        }
    }

    pub async fn save(state: Arc<Self>) -> Result<(), String> {
        let raw_args = SaveArgs {
            export_folder: state.export_folder.lock_ref().clone(),
            export_format: state.export_format.lock_ref().as_str().to_string(),
            template_folder: state.template_folder.lock_ref().clone(),
        };
        let args = tauri_wasm::args(&raw_args).map_err(|e| e.to_string())?;
        tauri_wasm::invoke("save_settings")
            .with_args(args)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

pub fn render(state: Arc<AppState>) -> Dom {
    html!("div", {
        .child_signal(state.page.signal_ref(clone!(state => move |page| {
            Some(match page {
                AppPage::Home => pages::home::render(state.clone()),
                AppPage::Settings => pages::settings::render(state.clone()),
                AppPage::RegisterWorkday => pages::register_workday::render(state.clone()),
                AppPage::TemplateMaker => pages::template_maker::render(state.clone()),
                AppPage::CategoryManager => pages::category_manager::render(state.clone()),
            })
        })))
    })
}
