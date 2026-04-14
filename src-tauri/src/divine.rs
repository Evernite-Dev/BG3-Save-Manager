use std::path::{Path, PathBuf};
use std::collections::HashMap;
#[cfg(windows)]
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

// ── Path → Divine argument ────────────────────────────────────────────────────
//
// Divine's CLI uses .NET's System.Uri to validate paths. On Windows, a path
// like "C:\foo\bar" is recognised as an absolute file URI. On Linux, an
// absolute path "/home/deck/foo" is treated as a *relative* URI by .NET,
// causing "This operation is not supported for a relative URI". Prefixing
// with "file://" produces a proper absolute file URI on all platforms.

pub fn to_divine_arg(path: &std::path::Path) -> Option<String> {
    #[cfg(unix)]
    {
        path.to_str().map(|s| format!("file://{s}"))
    }
    #[cfg(not(unix))]
    {
        path.to_str().map(|s| s.to_string())
    }
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
        use tauri::Manager;
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

// ── Native LSPK reader (Linux / macOS) ───────────────────────────────────────
//
// Divine's CLI path validation is incompatible with Linux absolute paths on
// .NET 8: `new Uri("/path").IsFile` treats them as relative URIs, and the
// alternative `file:///path` format fails a subsequent `Path.IsPathRooted`
// check. We therefore read the .lsv package natively in Rust on non-Windows.
//
// BG3 saves use LSPK version 18. Layout:
//   [0..4]   magic "LSPK"
//   [4..8]   version (u32 LE) = 18
//   [8..16]  file-list offset (u64 LE)
//   [16..20] file-list compressed size (u32 LE)
//   [20..22] flags (u16)  [22..24] priority (u16)  [24..40] MD5  [40..42] num_parts
// At file-list offset:
//   [0..4]   num_files (u32 LE)
//   [4..8]   compressed size of file table (u32 LE)
//   [8..]    LZ4-block-compressed file table
// file_list_size in header = 8 + compressed_size (covers all three sections)
// Each file entry (272 bytes, packed):
//   [0..256]   name (null-terminated UTF-8)
//   [256..260] offset_lo (u32 LE)
//   [260..262] offset_hi (u16 LE)  → file_offset = offset_lo | (offset_hi << 32)
//   [262]      archive_part
//   [263]      flags (low nibble = compression: 0=none, 2=LZ4, 3=LZ4HC, 4=Zstd)
//   [264..268] size_on_disk (u32 LE)
//   [268..272] uncompressed_size (u32 LE)

#[cfg(not(windows))]
fn read_save_info_json_native(lsv_path: &Path) -> Option<SaveInfoJson> {
    use std::io::{Read, Seek, SeekFrom};

    let mut f = std::fs::File::open(lsv_path)
        .map_err(|e| eprintln!("[lspk] open {:?}: {e}", lsv_path)).ok()?;

    // Validate magic + version
    let mut sig = [0u8; 4];
    f.read_exact(&mut sig).ok()?;
    eprintln!("[lspk] magic={:?}", std::str::from_utf8(&sig).unwrap_or("?"));
    if &sig != b"LSPK" { eprintln!("[lspk] bad magic"); return None; }
    let version = read_u32_le(&mut f)?;
    eprintln!("[lspk] version={version}");
    if version != 18 { eprintln!("[lspk] bad version"); return None; }

    let file_list_offset = read_u64_le(&mut f)?;
    let _file_list_size  = read_u32_le(&mut f)?;
    eprintln!("[lspk] file_list_offset={file_list_offset}");

    // Read file table: [num_files (u32)][compressed_size (u32)][LZ4 data]
    f.seek(SeekFrom::Start(file_list_offset)).ok()?;
    let num_files       = read_u32_le(&mut f)? as usize;
    let compressed_size = read_u32_le(&mut f)? as usize;
    eprintln!("[lspk] num_files={num_files} compressed_size={compressed_size}");

    let mut compressed = vec![0u8; compressed_size];
    f.read_exact(&mut compressed)
        .map_err(|e| eprintln!("[lspk] read compressed: {e}")).ok()?;

    const ENTRY: usize = 272;
    eprintln!("[lspk] decompressing {} -> expected {}", compressed_size, num_files * ENTRY);
    let table = lz4_flex::decompress(&compressed, num_files * ENTRY)
        .map_err(|e| eprintln!("[lspk] lz4 decompress: {e}")).ok()?;
    eprintln!("[lspk] decompressed ok, scanning {} entries", num_files);

    for i in 0..num_files {
        let e = &table[i * ENTRY..(i + 1) * ENTRY];
        let nul = e[..256].iter().position(|&b| b == 0).unwrap_or(256);
        let name = std::str::from_utf8(&e[..nul]).unwrap_or("?");
        if name != "SaveInfo.json" { continue; }
        eprintln!("[lspk] found SaveInfo.json at entry {i}");

        let offset_lo = u32::from_le_bytes(e[256..260].try_into().ok()?) as u64;
        let offset_hi = u16::from_le_bytes(e[260..262].try_into().ok()?) as u64;
        let flags         = e[263];
        let size_on_disk  = u32::from_le_bytes(e[264..268].try_into().ok()?) as usize;
        let uncomp_size   = u32::from_le_bytes(e[268..272].try_into().ok()?) as usize;
        let file_offset   = offset_lo | (offset_hi << 32);
        eprintln!("[lspk] SaveInfo.json: offset={file_offset} size_on_disk={size_on_disk} uncomp={uncomp_size} flags={flags:#04x}");

        f.seek(SeekFrom::Start(file_offset))
            .map_err(|e| eprintln!("[lspk] seek: {e}")).ok()?;
        let mut data = vec![0u8; size_on_disk];
        f.read_exact(&mut data)
            .map_err(|e| eprintln!("[lspk] read data: {e}")).ok()?;

        let json_bytes: Vec<u8> = match flags & 0x0F {
            0       => { eprintln!("[lspk] no compression"); data }
            2 | 3   => {
                // Individual files use LZ4 frame format (not raw block)
                use std::io::Read;
                let mut dec = lz4_flex::frame::FrameDecoder::new(
                    std::io::Cursor::new(&data));
                let mut out = Vec::with_capacity(uncomp_size);
                dec.read_to_end(&mut out)
                    .map_err(|e| eprintln!("[lspk] lz4 frame: {e}")).ok()?;
                out
            }
            4       => zstd::bulk::decompress(&data, uncomp_size)
                            .map_err(|e| eprintln!("[lspk] zstd json: {e}")).ok()?,
            other   => { eprintln!("[lspk] unknown compression flag {other}"); return None; }
        };

        eprintln!("[lspk] json {} bytes, parsing", json_bytes.len());
        let result = serde_json::from_slice(&json_bytes)
            .map_err(|e| eprintln!("[lspk] json parse: {e}")).ok();
        eprintln!("[lspk] parse result: {}", if result.is_some() { "ok" } else { "NONE" });
        return result;
    }
    eprintln!("[lspk] SaveInfo.json not found in file table");
    None
}

#[cfg(not(windows))]
fn read_u32_le(f: &mut std::fs::File) -> Option<u32> {
    use std::io::Read;
    let mut b = [0u8; 4]; f.read_exact(&mut b).ok()?; Some(u32::from_le_bytes(b))
}
#[cfg(not(windows))]
fn read_u64_le(f: &mut std::fs::File) -> Option<u64> {
    use std::io::Read;
    let mut b = [0u8; 8]; f.read_exact(&mut b).ok()?; Some(u64::from_le_bytes(b))
}

// ── Core extraction ───────────────────────────────────────────────────────────

/// Extracts SaveInfo.json from an .lsv package, parses it, and returns a
/// `SaveSummary`.  On non-Windows reads the package natively (avoids Divine's
/// broken Linux path validation).  Returns `None` on any failure.
pub fn extract_save_info(app: &tauri::AppHandle, save_folder: &Path) -> Option<SaveSummary> {
    let lsv = save_folder.join("HonourMode.lsv");
    if !lsv.exists() {
        return None;
    }

    // ── Parse SaveInfo.json ──────────────────────────────────────────────────
    #[cfg(not(windows))]
    let info: SaveInfoJson = { let _ = app; read_save_info_json_native(&lsv)? };

    #[cfg(windows)]
    let info: SaveInfoJson = {
        let temp_dir = std::env::temp_dir().join(format!(
            "bg3_inspect_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::create_dir_all(&temp_dir);

        let divine = divine_path(app);
        let result = (|| -> Option<SaveInfoJson> {
            let lsv_arg = to_divine_arg(&lsv)?;
            let tmp_arg = to_divine_arg(&temp_dir)?;
            let mut cmd = Command::new(&divine);
            cmd.args(["-g", "bg3", "-a", "extract-package",
                      "-s", &lsv_arg, "-d", &tmp_arg]);
            cmd.creation_flags(CREATE_NO_WINDOW);
            if !cmd.status().ok()?.success() { return None; }
            let json_str = std::fs::read_to_string(temp_dir.join("SaveInfo.json")).ok()?;
            serde_json::from_str(&json_str).ok()
        })();
        let _ = std::fs::remove_dir_all(&temp_dir);
        result?
    };

    // ── Build summary from parsed JSON ───────────────────────────────────────
    {
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
    }
}
