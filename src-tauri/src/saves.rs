use std::path::{Path, PathBuf};
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use tauri::AppHandle;

use crate::divine::{self, SaveSummary};
use crate::paths;

// ── Public data types sent to the frontend ────────────────────────────────────

#[derive(serde::Serialize, Clone)]
pub struct RunInfo {
    pub folder_name: String,
    pub full_path:   String,
    pub summary:     Option<SaveSummary>,
}

#[derive(serde::Serialize, Clone)]
pub struct BackupInfo {
    pub folder_name: String,
    pub display:     String,
    pub label:       String,
    pub date:        String,
    pub summary:     Option<SaveSummary>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn honour_saves(save_dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(save_dir) else {
        return vec![];
    };
    let mut saves: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with("__HonourMode"))
                .unwrap_or(false)
        })
        .collect();
    saves.sort();
    saves
}

fn backups_for_run(backup_dir: &Path, save_folder_name: &str) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(backup_dir) else {
        return vec![];
    };
    let suffix = format!("_{}", save_folder_name);
    let mut backups: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(&suffix))
                .unwrap_or(false)
        })
        .collect();
    // Newest first
    backups.sort_by(|a, b| b.cmp(a));
    backups
}

/// Parse a backup folder name like `2026-04-13_14-38-32_[Label]_<save_name>`
/// into (date_display, label).
fn parse_backup_name(name: &str) -> (String, String) {
    if name.len() < 19 {
        return (name.to_string(), String::new());
    }
    let date = &name[..10];
    let time = name[11..19].replace('-', ":");
    let after = &name[20..];

    let label = if let Some(rest) = after.strip_prefix('[') {
        rest.find(']').map(|i| rest[..i].to_string()).unwrap_or_default()
    } else {
        String::new()
    };

    (format!("{date} {time}"), label)
}

fn load_summary(folder: &Path) -> Option<SaveSummary> {
    let path = folder.join("summary.json");
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn save_summary(folder: &Path, summary: &SaveSummary) {
    if let Ok(json) = serde_json::to_string_pretty(summary) {
        let _ = std::fs::write(folder.join("summary.json"), json);
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_save_dir_path(_app: AppHandle) -> Option<String> {
    paths::save_dir().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_backup_dir_path(app: AppHandle) -> String {
    let dir = paths::backup_dir(&app);
    dir.to_string_lossy().to_string()
}

#[tauri::command]
pub async fn open_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_honour_saves(app: AppHandle) -> Vec<RunInfo> {
    let Some(save_dir) = paths::save_dir() else {
        return vec![];
    };

    tokio::task::spawn_blocking(move || {
        honour_saves(&save_dir)
            .into_iter()
            .map(|path| {
                let folder_name = path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                let summary = divine::extract_save_info(&app, &path);
                RunInfo {
                    full_path: path.to_string_lossy().to_string(),
                    folder_name,
                    summary,
                }
            })
            .collect()
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
pub async fn get_backups_for_run(
    app: AppHandle,
    save_folder_name: String,
) -> Vec<BackupInfo> {
    let backup_dir = paths::backup_dir(&app);

    tokio::task::spawn_blocking(move || {
        backups_for_run(&backup_dir, &save_folder_name)
            .into_iter()
            .map(|path| {
                let folder_name = path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                let (date, label) = parse_backup_name(&folder_name);
                let summary = load_summary(&path);

                let display = if let Some(ref s) = summary {
                    if label.is_empty() {
                        format!("{date}  {}  |  {}", s.display_name, s.location)
                    } else {
                        format!("{date}  [{}]  {}  |  {}", label, s.display_name, s.location)
                    }
                } else if label.is_empty() {
                    format!("{date}  {folder_name}")
                } else {
                    format!("{date}  [{label}]  {folder_name}")
                };

                BackupInfo { folder_name, display, label, date, summary }
            })
            .collect()
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
pub async fn backup_save(
    app: AppHandle,
    save_folder: String,
    label: String,
) -> Result<String, String> {
    let backup_dir = paths::backup_dir(&app);
    let app2 = app.clone();

    tokio::task::spawn_blocking(move || {
        let save_path = PathBuf::from(&save_folder);
        let lsv = save_path.join("HonourMode.lsv");
        if !lsv.exists() {
            return Err("HonourMode.lsv not found in save folder.".to_string());
        }

        let save_name = save_path
            .file_name()
            .ok_or("Invalid save folder path")?
            .to_string_lossy();

        let timestamp = chrono_now();
        let dest_name = if label.is_empty() {
            format!("{timestamp}_{save_name}")
        } else {
            format!("{timestamp}_[{label}]_{save_name}")
        };

        std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
        let dest = backup_dir.join(&dest_name);

        copy_dir_all(&save_path, &dest).map_err(|e| e.to_string())?;

        // Generate summary.json from the backed-up copy
        if let Some(summary) = divine::extract_save_info(&app2, &save_path) {
            save_summary(&dest, &summary);
        }

        Ok(format!("Backed up to: {dest_name}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn restore_save(
    app: AppHandle,
    backup_name: String,
    save_name: String,
) -> Result<String, String> {
    let backup_dir = paths::backup_dir(&app);
    let Some(save_dir) = paths::save_dir() else {
        return Err("Could not locate BG3 save directory.".to_string());
    };
    let app2 = app.clone();

    tokio::task::spawn_blocking(move || {
        let target = save_dir.join(&save_name);
        let source = backup_dir.join(&backup_name);

        // Auto-backup the current save before overwriting
        if target.exists() {
            let timestamp = chrono_now();
            let auto_name = format!("{timestamp}_[pre-restore]_{save_name}");
            let auto_dest = backup_dir.join(&auto_name);
            std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
            copy_dir_all(&target, &auto_dest).map_err(|e| e.to_string())?;
            if let Some(summary) = divine::extract_save_info(&app2, &target) {
                save_summary(&auto_dest, &summary);
            }
        }

        // Clear destination and copy backup in (skip summary.json)
        if target.exists() {
            for entry in std::fs::read_dir(&target).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let _ = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    std::fs::remove_dir_all(entry.path())
                } else {
                    std::fs::remove_file(entry.path())
                };
            }
        } else {
            std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;
        }

        for entry in std::fs::read_dir(&source).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_name() == "summary.json" {
                continue;
            }
            let dst = target.join(entry.file_name());
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                copy_dir_all(&entry.path(), &dst).map_err(|e| e.to_string())?;
            } else {
                std::fs::copy(entry.path(), dst).map_err(|e| e.to_string())?;
            }
        }

        Ok(format!("Restored '{backup_name}' successfully."))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn delete_backup(
    app: AppHandle,
    backup_name: String,
) -> Result<(), String> {
    let backup_dir = paths::backup_dir(&app);

    tokio::task::spawn_blocking(move || {
        let path = backup_dir.join(&backup_name);
        std::fs::remove_dir_all(path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_backup_image(backup_path: String) -> Option<String> {
    tokio::task::spawn_blocking(move || {
        let dir = PathBuf::from(&backup_path);
        // Find the first WebP file (case-insensitive extension)
        let webp = std::fs::read_dir(&dir)
            .ok()?
            .flatten()
            .find(|e| {
                e.path()
                    .extension()
                    .and_then(|x| x.to_str())
                    .map(|x| x.eq_ignore_ascii_case("webp"))
                    .unwrap_or(false)
            })?
            .path();

        let bytes = std::fs::read(webp).ok()?;
        Some(B64.encode(bytes))
    })
    .await
    .ok()
    .flatten()
}

#[tauri::command]
pub async fn backup_all_saves(
    app: AppHandle,
    label: String,
) -> Result<String, String> {
    let Some(save_dir) = paths::save_dir() else {
        return Err("Could not locate BG3 save directory.".to_string());
    };
    let backup_dir = paths::backup_dir(&app);
    let app2 = app.clone();

    tokio::task::spawn_blocking(move || {
        let saves = honour_saves(&save_dir);
        if saves.is_empty() {
            return Err("No HonourMode saves found.".to_string());
        }

        std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
        let count = saves.len();
        let timestamp = chrono_now();

        for save_path in saves {
            let save_name = save_path
                .file_name()
                .unwrap()
                .to_string_lossy();
            let dest_name = if label.is_empty() {
                format!("{timestamp}_{save_name}")
            } else {
                format!("{timestamp}_[{label}]_{save_name}")
            };
            let dest = backup_dir.join(&dest_name);
            copy_dir_all(&save_path, &dest).map_err(|e| e.to_string())?;
            if let Some(summary) = divine::extract_save_info(&app2, &save_path) {
                save_summary(&dest, &summary);
            }
        }

        Ok(format!("Backed up {count} run(s)."))
    })
    .await
    .map_err(|e| e.to_string())?
}

// ── Timestamp helper ──────────────────────────────────────────────────────────

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Manual conversion — avoids the chrono crate dependency.
    let (y, mo, d, h, mi, s) = epoch_to_parts(secs);
    format!("{y:04}-{mo:02}-{d:02}_{h:02}-{mi:02}-{s:02}")
}

fn epoch_to_parts(mut secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let s  = secs % 60; secs /= 60;
    let mi = secs % 60; secs /= 60;
    let h  = secs % 24; secs /= 24;

    // Days since 1970-01-01
    let mut days = secs;
    let mut y = 1970u64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if days < days_in_year { break; }
        days -= days_in_year;
        y += 1;
    }
    let months = [31, if is_leap(y) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut mo = 1u64;
    for &m in &months {
        if days < m { break; }
        days -= m;
        mo += 1;
    }
    (y, mo, days + 1, h, mi, s)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}
