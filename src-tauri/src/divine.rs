use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::process::Command;
use serde::Deserialize;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// ── JSON structures from SaveInfo.json (produced by Divine) ──────────────────

#[derive(Deserialize)]
struct SaveInfoJson {
    #[serde(rename = "Current Level")]
    current_level: Option<String>,
    #[serde(rename = "Active Party")]
    active_party: Option<ActiveParty>,
}

#[derive(Deserialize)]
struct ActiveParty {
    #[serde(rename = "Characters")]
    characters: Option<Vec<Character>>,
}

#[derive(Deserialize)]
struct Character {
    #[serde(rename = "Origin")]
    origin: Option<String>,
    #[serde(rename = "Race")]
    race: Option<String>,
    #[serde(rename = "Level")]
    level: Option<serde_json::Value>,
    #[serde(rename = "Classes")]
    classes: Option<Vec<ClassInfo>>,
}

#[derive(Deserialize)]
struct ClassInfo {
    #[serde(rename = "Main")]
    main: Option<String>,
}

// ── Public summary type ───────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SaveSummary {
    pub display_name: String,
    pub location:     String,
    pub companions:   String,
    pub level:        u32,
    pub classes:      String,
    pub race:         String,
    pub party_size:   u32,
}

// ── Path resolution ───────────────────────────────────────────────────────────

pub fn divine_path(app: &tauri::AppHandle) -> PathBuf {
    // Dev builds: always resolve from src-tauri/binaries/ directly.
    // This ensures all sibling DLLs (required by the .NET host) are present.
    #[cfg(debug_assertions)]
    {
        let _ = app; // suppress unused warning
        #[cfg(windows)]
        return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries").join("Divine.exe");
        #[cfg(not(windows))]
        return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries").join("Divine");
    }

    // Release builds: all of binaries/ is bundled into the resource directory.
    #[cfg(not(debug_assertions))]
    {
        #[cfg(windows)]
        {
            let resource_dir = app.path().resource_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            return resource_dir.join("binaries").join("Divine.exe");
        }

        #[cfg(not(windows))]
        {
            // Flatpak: binaries are installed to /app/share/{app-id}/binaries/
            if let Ok(id) = std::env::var("FLATPAK_ID") {
                return PathBuf::from(format!("/app/share/{}/binaries/Divine", id));
            }
            // AppImage / regular install
            let resource_dir = app.path().resource_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            return resource_dir.join("binaries").join("Divine");
        }
    }
}

// ── Race / location display maps ──────────────────────────────────────────────

fn race_map() -> HashMap<&'static str, &'static str> {
    [
        ("Gnome_Deep",                    "Deep Gnome"),
        ("Gnome_RockGnome",               "Rock Gnome"),
        ("Gnome_ForestGnome",             "Forest Gnome"),
        ("Elf_HighElf",                   "High Elf"),
        ("Elf_WoodElf",                   "Wood Elf"),
        ("Drow_LolthSworn",               "Drow (Lolth-Sworn)"),
        ("Drow_Seldarine",                "Drow (Seldarine)"),
        ("Human",                         "Human"),
        ("HalfElf_HighHalfElf",           "High Half-Elf"),
        ("HalfElf_WoodHalfElf",           "Wood Half-Elf"),
        ("Halfling_LightfootHalfling",     "Lightfoot Halfling"),
        ("Halfling_StoutHalfling",         "Stout Halfling"),
        ("Tiefling_Asmodeus",             "Asmodeus Tiefling"),
        ("Tiefling_Mephistopheles",       "Mephistopheles Tiefling"),
        ("Tiefling_Zariel",               "Zariel Tiefling"),
        ("Dwarf_GoldDwarf",               "Gold Dwarf"),
        ("Dwarf_ShieldDwarf",             "Shield Dwarf"),
        ("Githyanki",                     "Githyanki"),
        ("HalfOrc",                       "Half-Orc"),
    ]
    .into_iter()
    .collect()
}

fn location_map() -> HashMap<&'static str, &'static str> {
    [
        ("WLD_Main_A",          "Act 1 – Wilderness"),
        ("UND_Main_A",          "Act 1 – Underdark"),
        ("UND_Grymforge_A",     "Act 1 – Grymforge"),
        ("SCL_Main_A",          "Act 1 – Shattered Sanctum"),
        ("SHC_Main_A",          "Act 2 – Shadow-Cursed Lands"),
        ("SHC_Moonrise_A",      "Act 2 – Moonrise Towers"),
        ("GTY_Main_A",          "Act 2 – Gauntlet of Shar"),
        ("BGO_Main_A",          "Act 3 – Outer City"),
        ("CTY_Main_A",          "Act 3 – Lower City"),
        ("IRN_Main_A",          "Act 3 – Iron Throne"),
        ("IRN_Submersible_A",   "Act 3 – Submersible"),
        ("CRE_AstralPlane_E_Art", "Endgame – Astral Plane"),
        ("SYS_CC_I",            "Character Creation"),
    ]
    .into_iter()
    .collect()
}

fn fmt_race(raw: &str) -> String {
    race_map()
        .get(raw)
        .map(|s| s.to_string())
        .unwrap_or_else(|| raw.replace('_', " "))
}

fn fmt_location(raw: &str) -> String {
    location_map()
        .get(raw)
        .map(|s| s.to_string())
        .unwrap_or_else(|| raw.to_string())
}

fn parse_level(v: &serde_json::Value) -> u32 {
    match v {
        serde_json::Value::Number(n) => n.as_u64().unwrap_or(0) as u32,
        serde_json::Value::String(s) => s.parse().unwrap_or(0),
        _ => 0,
    }
}

// ── Core extraction ───────────────────────────────────────────────────────────

/// Extracts SaveInfo.json from an .lsv package using Divine, parses it, and
/// returns a `SaveSummary`.  Returns `None` on any failure.
pub fn extract_save_info(app: &tauri::AppHandle, save_folder: &Path) -> Option<SaveSummary> {
    let lsv = save_folder.join("HonourMode.lsv");
    if !lsv.exists() {
        return None;
    }

    let temp_dir = std::env::temp_dir().join(format!(
        "bg3_inspect_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::create_dir_all(&temp_dir);

    let divine = divine_path(app);

    let result = (|| -> Option<SaveSummary> {
        let mut cmd = Command::new(&divine);
        cmd.args([
            "-g", "bg3",
            "-a", "extract-package",
            "-s", lsv.to_str()?,
            "-d", temp_dir.to_str()?,
        ]);
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        let status = cmd.status().ok()?;

        if !status.success() {
            return None;
        }

        let json_path = temp_dir.join("SaveInfo.json");
        let json_str = std::fs::read_to_string(&json_path).ok()?;
        let info: SaveInfoJson = serde_json::from_str(&json_str).ok()?;

        let chars = info.active_party?.characters.unwrap_or_default();

        // Prefer the custom (Generic origin) character; fall back to first entry.
        let player = chars
            .iter()
            .find(|c| c.origin.as_deref() == Some("Generic"))
            .or_else(|| chars.first())?;

        let race = player.race.as_deref().map(fmt_race).unwrap_or_default();
        let level = player.level.as_ref().map(parse_level).unwrap_or(0);
        let classes = player
            .classes
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .filter_map(|c| c.main.as_deref())
            .collect::<Vec<_>>()
            .join("/");

        let companions = chars
            .iter()
            .filter(|c| c.origin.as_deref() != Some("Generic"))
            .filter_map(|c| c.origin.as_deref())
            .collect::<Vec<_>>()
            .join(", ");

        let location = info
            .current_level
            .as_deref()
            .map(fmt_location)
            .unwrap_or_default();

        let display_name = format!("{race} {classes} Lv.{level}");

        Some(SaveSummary {
            display_name,
            location,
            companions,
            level,
            classes,
            race,
            party_size: chars.len() as u32,
        })
    })();

    let _ = std::fs::remove_dir_all(&temp_dir);
    result
}
