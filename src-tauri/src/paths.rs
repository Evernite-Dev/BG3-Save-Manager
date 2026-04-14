use std::path::PathBuf;

/// Returns the BG3 save directory for the current OS.
/// Windows : %LOCALAPPDATA%\Larian Studios\Baldur's Gate 3\PlayerProfiles\Public\Savegames\Story
/// macOS   : ~/Library/Application Support/Larian Studios/Baldur's Gate 3/…
/// Linux   : ~/.local/share/Larian Studios/Baldur's Gate 3/… (native Steam)
///           then falls back to the Steam Proton compatdata path.
pub fn save_dir() -> Option<PathBuf> {
    let bg3_tail = "Larian Studios/Baldur's Gate 3/PlayerProfiles/Public/Savegames/Story";

    #[cfg(target_os = "windows")]
    {
        dirs::data_local_dir().map(|p| p.join(bg3_tail))
    }

    #[cfg(target_os = "macos")]
    {
        dirs::data_dir().map(|p| p.join(bg3_tail))
    }

    #[cfg(target_os = "linux")]
    {
        // 1. Native Steam Linux path
        let native = dirs::data_local_dir().map(|p| p.join(bg3_tail));
        if let Some(ref p) = native {
            if p.exists() {
                return native;
            }
        }

        // 2. Steam Proton compatdata path (app ID 1086940)
        let proton = dirs::home_dir().map(|h| {
            h.join(".steam/steam/steamapps/compatdata/1086940/pfx/drive_c/users/steamuser")
             .join("AppData/Local")
             .join(bg3_tail)
        });
        if let Some(ref p) = proton {
            if p.exists() {
                return proton;
            }
        }

        // 3. Return native guess even if it doesn't exist yet
        native
    }
}

/// Returns the application backup storage directory.
/// This lives in the OS app-local data folder so it survives app reinstalls.
pub fn backup_dir(app: &tauri::AppHandle) -> PathBuf {
    use tauri::Manager;
    app.path()
        .app_local_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("Save Backups")
}

/// Returns the BG3 PlayerProfiles/Public directory (one level above the save dir).
/// This is where profile8.lsf lives.
pub fn profile_dir() -> Option<PathBuf> {
    let bg3_base = "Larian Studios/Baldur's Gate 3/PlayerProfiles/Public";

    #[cfg(target_os = "windows")]
    { dirs::data_local_dir().map(|p| p.join(bg3_base)) }

    #[cfg(target_os = "macos")]
    { dirs::data_dir().map(|p| p.join(bg3_base)) }

    #[cfg(target_os = "linux")]
    {
        let native = dirs::data_local_dir().map(|p| p.join(bg3_base));
        if let Some(ref p) = native { if p.exists() { return native; } }
        let proton = dirs::home_dir().map(|h| {
            h.join(".steam/steam/steamapps/compatdata/1086940/pfx/drive_c/users/steamuser")
             .join("AppData/Local")
             .join(bg3_base)
        });
        if let Some(ref p) = proton { if p.exists() { return proton; } }
        native
    }
}

/// Working directory for the profile editor (stores backups and converted files).
pub fn profile_work_dir(app: &tauri::AppHandle) -> PathBuf {
    use tauri::Manager;
    app.path()
        .app_local_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("ProfileEditor")
}
