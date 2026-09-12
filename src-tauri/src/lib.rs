mod backup;
mod bangumi;
mod commands;
mod covers;
mod db;
mod error;
mod models;
mod stats;
mod validate;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let state = commands::init_state(app.handle());
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::list_library,
            commands::get_library_entry,
            commands::add_library_entry,
            commands::update_personal_record,
            commands::list_tiers,
            commands::save_tier,
            commands::reorder_tiers,
            commands::delete_tier,
            commands::search_subjects,
            commands::get_subject,
            commands::list_recent_searches,
            commands::refresh_subject,
            commands::get_statistics,
            commands::export_backup,
            commands::preview_import,
            commands::import_backup,
            commands::clear_cover_cache,
            commands::delete_personal_data,
            commands::snapshot_database,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


