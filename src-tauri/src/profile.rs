use std::path::PathBuf;
use std::process::Command;
use tauri::AppHandle;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
const CREATE_NO_WINDOW: u32 = 0x08000000;

use crate::divine::divine_path;
use crate::paths;

// ── Internal path helpers ─────────────────────────────────────────────────────

fn source_lsf() -> Option<PathBuf> {
    paths::profile_dir().map(|d| d.join("profile8.lsf"))
}

/// Staged working copy — named profile8.lsf so Divine infers the format.
fn work_lsf(app: &AppHandle) -> PathBuf {
    paths::profile_work_dir(app).join("profile8.lsf")
}

/// Original backup kept throughout the session.
fn backup_lsf(app: &AppHandle) -> PathBuf {
    paths::profile_work_dir(app).join("profile8_original.lsf")
}

/// Converted LSX that is read and edited.
fn work_lsx(app: &AppHandle) -> PathBuf {
    paths::profile_work_dir(app).join("profile8.lsx")
}

/// Edited LSF ready to overwrite the live file.
fn edited_lsf(app: &AppHandle) -> PathBuf {
    paths::profile_work_dir(app).join("profile8_edited.lsf")
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Find profile8.lsf, back it up, and stage a working copy.
#[tauri::command]
pub async fn load_profile(app: AppHandle) -> Result<String, String> {
    let source = source_lsf()
        .ok_or_else(|| "Could not locate BG3 PlayerProfiles directory.".to_string())?;

    if !source.exists() {
        return Err(format!("profile8.lsf not found at: {}", source.display()));
    }

    let work_dir = paths::profile_work_dir(&app);
    std::fs::create_dir_all(&work_dir).map_err(|e| e.to_string())?;

    std::fs::copy(&source, backup_lsf(&app)).map_err(|e| e.to_string())?;
    std::fs::copy(&source, work_lsf(&app)).map_err(|e| e.to_string())?;

    Ok(format!("Loaded and backed up: {}", source.display()))
}

/// Convert the staged profile8.lsf → profile8.lsx using Divine.
/// Divine infers formats from file extensions.
#[tauri::command]
pub async fn prepare_profile(app: AppHandle) -> Result<String, String> {
    let src = work_lsf(&app);
    let dst = work_lsx(&app);

    if !src.exists() {
        return Err("Load the profile first.".to_string());
    }

    let divine = divine_path(&app);

    tokio::task::spawn_blocking(move || {
        let mut cmd = Command::new(&divine);
        cmd.args([
            "-g", "bg3",
            "-a", "convert-resource",
            "-s", src.to_str().unwrap_or_default(),
            "-d", dst.to_str().unwrap_or_default(),
            "--input-format", "lsf",
            "--output-format", "lsx",
        ]);
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        let out = cmd.output().map_err(|e| format!("Failed to launch Divine: {e}"))?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let stdout = String::from_utf8_lossy(&out.stdout);
            let detail = [stderr.trim(), stdout.trim()]
                .iter()
                .filter(|s| !s.is_empty())
                .cloned()
                .collect::<Vec<_>>()
                .join(" | ");
            return Err(if detail.is_empty() {
                "Divine LSF→LSX conversion failed (no output).".to_string()
            } else {
                format!("Divine LSF→LSX failed: {detail}")
            });
        }

        Ok("Profile converted to LSX.".to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Read and return the current profile8.lsx content.
#[tauri::command]
pub async fn get_profile_content(app: AppHandle) -> Result<String, String> {
    let lsx = work_lsx(&app);
    tokio::task::spawn_blocking(move || {
        std::fs::read_to_string(&lsx)
            .map_err(|e| format!("Could not read profile LSX: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Strip every <node id="DisabledSingleSaveSessions">…</node> block,
/// save the result in-place, and return the modified content.
#[tauri::command]
pub async fn remove_fail_flags(app: AppHandle) -> Result<String, String> {
    let lsx = work_lsx(&app);

    tokio::task::spawn_blocking(move || {
        let content = std::fs::read_to_string(&lsx)
            .map_err(|e| format!("Could not read profile LSX: {e}"))?;

        let modified = strip_disabled_sessions(&content);

        if modified == content {
            return Err("No DisabledSingleSaveSessions nodes found — nothing to remove.".to_string());
        }

        std::fs::write(&lsx, &modified)
            .map_err(|e| format!("Could not save modified profile: {e}"))?;

        Ok(modified)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Convert the edited profile8.lsx back to profile8_edited.lsf.
#[tauri::command]
pub async fn save_profile(app: AppHandle) -> Result<String, String> {
    let src = work_lsx(&app);
    let dst = edited_lsf(&app);

    if !src.exists() {
        return Err("Prepare the profile first.".to_string());
    }

    let divine = divine_path(&app);

    tokio::task::spawn_blocking(move || {
        let mut cmd = Command::new(&divine);
        cmd.args([
            "-g", "bg3",
            "-a", "convert-resource",
            "-s", src.to_str().unwrap_or_default(),
            "-d", dst.to_str().unwrap_or_default(),
            "--input-format", "lsx",
            "--output-format", "lsf",
        ]);
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        let out = cmd.output().map_err(|e| format!("Failed to launch Divine: {e}"))?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let stdout = String::from_utf8_lossy(&out.stdout);
            let detail = [stderr.trim(), stdout.trim()]
                .iter()
                .filter(|s| !s.is_empty())
                .cloned()
                .collect::<Vec<_>>()
                .join(" | ");
            return Err(if detail.is_empty() {
                "Divine LSX→LSF conversion failed (no output).".to_string()
            } else {
                format!("Divine LSX→LSF failed: {detail}")
            });
        }

        Ok("Edited profile saved as LSF.".to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Overwrite the live profile8.lsf with the edited version.
#[tauri::command]
pub async fn overwrite_profile(app: AppHandle) -> Result<String, String> {
    let src = edited_lsf(&app);
    let dst = source_lsf()
        .ok_or_else(|| "Could not locate BG3 PlayerProfiles directory.".to_string())?;

    if !src.exists() {
        return Err("Save the edited profile first.".to_string());
    }

    tokio::task::spawn_blocking(move || {
        std::fs::copy(&src, &dst)
            .map_err(|e| format!("Failed to overwrite profile: {e}"))?;
        Ok(format!("Profile overwritten: {}", dst.display()))
    })
    .await
    .map_err(|e| e.to_string())?
}

// ── XML helpers ───────────────────────────────────────────────────────────────

/// Remove every non-self-closing `<node id="DisabledSingleSaveSessions">…</node>` block.
/// Self-closing nodes (`<node id="DisabledSingleSaveSessions" />`) are left untouched —
/// they indicate no failures are recorded.
fn strip_disabled_sessions(input: &str) -> String {
    let marker = r#"<node id="DisabledSingleSaveSessions""#;
    let eol = if input.contains("\r\n") { "\r\n" } else { "\n" };

    let lines: Vec<&str> = if eol == "\r\n" {
        input.split("\r\n").collect()
    } else {
        input.split('\n').collect()
    };

    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim_start();

        // Only remove non-self-closing DisabledSingleSaveSessions nodes
        if trimmed.starts_with(marker) && !trimmed.trim_end().ends_with("/>") {
            // Skip all lines until (and including) the matching </node>
            let mut depth: i32 = 0;
            while i < lines.len() {
                let t = lines[i].trim_start();
                if (t.starts_with("<node ") || t == "<node>") && !t.trim_end().ends_with("/>") {
                    depth += 1;
                }
                if t.starts_with("</node>") {
                    depth -= 1;
                }
                i += 1;
                if depth <= 0 {
                    break;
                }
            }
        } else {
            out.push(lines[i]);
            i += 1;
        }
    }

    let mut result = out.join(eol);
    if input.ends_with('\n') && !result.ends_with('\n') {
        result.push_str(eol);
    }
    result
}
