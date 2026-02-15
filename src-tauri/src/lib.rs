mod commands;
mod db;
mod device;
mod error;
mod models;
mod scanner;
mod sync;

use std::sync::Mutex;
use rusqlite::Connection;
use tauri::Manager;

use db::schema;
use sync::progress::CancelToken;

fn init_database(app: &tauri::App) -> Result<Connection, Box<dyn std::error::Error>> {
    let app_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_dir)?;
    // Migrate legacy database filename
    let legacy_path = app_dir.join("music_sync.db");
    let db_path = app_dir.join("orchestra.db");
    if legacy_path.exists() && !db_path.exists() {
        std::fs::rename(&legacy_path, &db_path)?;
    }
    let conn = Connection::open(db_path)?;
    schema::run_migrations(&conn)?;
    Ok(conn)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let conn = init_database(app)?;
            app.manage(Mutex::new(conn));
            app.manage(Mutex::new(CancelToken::new()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::library::scan_directory,
            commands::library::get_library_tree,
            commands::library::search_library,
            commands::library::get_incomplete_tracks,
            commands::library::get_library_stats,
            commands::library::find_duplicates,
            commands::library::delete_duplicate_tracks,
            commands::profile::create_profile,
            commands::profile::get_profile,
            commands::profile::list_profiles,
            commands::profile::update_profile,
            commands::profile::delete_profile,
            commands::sync_cmd::compute_diff,
            commands::sync_cmd::execute_sync,
            commands::sync_cmd::cancel_sync,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::get_all_settings,
            commands::device_cmd::detect_volumes,
            commands::device_cmd::register_device,
            commands::device_cmd::list_devices,
            commands::device_cmd::delete_device,
            commands::device_cmd::set_device_artists,
            commands::device_cmd::set_device_albums,
            commands::device_cmd::compute_device_diff,
            commands::device_cmd::execute_device_sync,
            commands::device_cmd::eject_device,
            commands::device_cmd::list_artists,
            commands::device_cmd::list_albums,
            commands::metadata_cmd::get_track_artwork,
            commands::metadata_cmd::update_track_metadata,
            commands::playlist_cmd::create_playlist,
            commands::playlist_cmd::list_playlists,
            commands::playlist_cmd::get_playlist,
            commands::playlist_cmd::update_playlist,
            commands::playlist_cmd::delete_playlist,
            commands::playlist_cmd::add_tracks_to_playlist,
            commands::playlist_cmd::remove_tracks_from_playlist,
            commands::playlist_cmd::reorder_playlist,
            commands::playlist_cmd::export_playlist,
            commands::favorite_cmd::toggle_favorite,
            commands::favorite_cmd::is_favorite,
            commands::favorite_cmd::list_favorites,
            commands::favorite_cmd::list_all_favorites,
            commands::favorite_cmd::get_favorite_tracks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
