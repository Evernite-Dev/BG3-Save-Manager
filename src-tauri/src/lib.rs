mod divine;
mod paths;
mod profile;
mod saves;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            use tauri::Manager;
            use image::GenericImageView;
            if let Some(window) = app.get_webview_window("main") {
                let bytes = include_bytes!("../icons/icon.png");
                let img = image::load_from_memory(bytes)?;
                let (w, h) = img.dimensions();
                let rgba = img.into_rgba8().into_raw();
                let icon = tauri::image::Image::new_owned(rgba, w, h);
                let _ = window.set_icon(icon);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            saves::get_save_dir_path,
            saves::get_backup_dir_path,
            saves::open_folder,
            saves::get_honour_saves,
            saves::get_backups_for_run,
            saves::backup_save,
            saves::backup_all_saves,
            saves::restore_save,
            saves::delete_backup,
            saves::get_backup_image,
            profile::load_profile,
            profile::prepare_profile,
            profile::get_profile_content,
            profile::remove_fail_flags,
            profile::save_profile,
            profile::overwrite_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
