mod categories;
mod dialog;
mod export;
mod settings;
mod templates;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            settings::get_settings,
            settings::save_settings,
            dialog::pick_folder,
            export::export_workday,
            export::export_monthly,
            templates::save_template,
            templates::list_templates,
            templates::delete_template,
            categories::get_categories,
            categories::save_categories,
            dialog::pick_categories_file,
            categories::import_categories,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
