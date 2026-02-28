//! Auto-Detection von Heightmap und Map-Mod-ZIP nach dem Laden einer XML-Datei.
//!
//! Prüft nach dem Laden einer AutoDrive-Config, ob:
//! 1. Eine `terrain.heightmap.png` im selben Verzeichnis liegt → direkt als Heightmap setzen
//! 2. Im Mods-Verzeichnis ein passendes ZIP zum `map_name` existiert → Dialog anzeigen

use regex::Regex;
use std::path::{Path, PathBuf};

/// Ergebnis der Auto-Detection nach dem Laden einer XML-Datei.
#[derive(Debug, Default)]
pub struct PostLoadDetectionResult {
    /// Pfad zur gefundenen Heightmap (falls vorhanden)
    pub heightmap_path: Option<PathBuf>,
    /// Pfad zu einer gefundenen overview.png im XML-Verzeichnis
    pub overview_path: Option<PathBuf>,
    /// Passende ZIP-Dateien im Mods-Verzeichnis
    pub matching_zips: Vec<PathBuf>,
}

/// Führt die komplette Auto-Detection durch.
///
/// Sucht nach `terrain.heightmap.png` im XML-Verzeichnis und nach passenden
/// Map-Mod-ZIPs im Mods-Verzeichnis (basierend auf `map_name`).
pub fn detect_post_load(xml_path: &Path, map_name: Option<&str>) -> PostLoadDetectionResult {
    let heightmap_path = find_heightmap_next_to(xml_path);
    let overview_path = find_overview_next_to(xml_path);
    let matching_zips = match (map_name, resolve_mods_dir(xml_path)) {
        (Some(name), Some(mods_dir)) if !name.is_empty() => find_matching_zips(&mods_dir, name),
        _ => Vec::new(),
    };
    PostLoadDetectionResult {
        heightmap_path,
        overview_path,
        matching_zips,
    }
}

/// Prüft ob `overview.jpg` (oder `overview.png`) im selben Verzeichnis wie die XML liegt.
fn find_overview_next_to(xml_path: &Path) -> Option<PathBuf> {
    let dir = xml_path.parent()?;
    // Bevorzugt .jpg, Fallback auf .png
    for name in &["overview.jpg", "overview.png"] {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Prüft ob `terrain.heightmap.png` im selben Verzeichnis wie die XML liegt.
fn find_heightmap_next_to(xml_path: &Path) -> Option<PathBuf> {
    let dir = xml_path.parent()?;
    let heightmap = dir.join("terrain.heightmap.png");
    if heightmap.is_file() {
        Some(heightmap)
    } else {
        None
    }
}

/// Ermittelt das Mods-Verzeichnis relativ zum XML-Pfad.
///
/// Erwartet die Savegame-Struktur:
/// `.../FarmingSimulator2025/savegameN/AutoDrive_config.xml`
/// → Mods-Verzeichnis: `.../FarmingSimulator2025/mods/`
fn resolve_mods_dir(xml_path: &Path) -> Option<PathBuf> {
    // xml_path.parent() = savegameN/
    // .parent() = FarmingSimulator2025/
    let savegame_dir = xml_path.parent()?;
    let fs_root = savegame_dir.parent()?;
    let mods_dir = fs_root.join("mods");
    if mods_dir.is_dir() {
        Some(mods_dir)
    } else {
        None
    }
}

/// Erzeugt Umlaut-Varianten eines Namens (bidirektional).
///
/// Ersetzt ä↔ae, ö↔oe, ü↔ue, ß↔ss in beide Richtungen.
/// Gibt alle eindeutigen Varianten zurück (inkl. Original).
fn expand_umlaut_variants(name: &str) -> Vec<String> {
    let lower = name.to_lowercase();

    // Richtung 1: Umlaute → ASCII-Digraphen
    let ascii = lower
        .replace('ä', "ae")
        .replace('ö', "oe")
        .replace('ü', "ue")
        .replace('ß', "ss");

    // Richtung 2: ASCII-Digraphen → Umlaute
    let umlaut = lower
        .replace("ae", "ä")
        .replace("oe", "ö")
        .replace("ue", "ü")
        .replace("ss", "ß");

    let mut variants = vec![lower];
    if !variants.contains(&ascii) {
        variants.push(ascii);
    }
    if !variants.contains(&umlaut) {
        variants.push(umlaut);
    }
    variants
}

/// Wandelt eine Namensvariante in ein Regex-Pattern um.
///
/// Spaces und Underscores werden als Wildcard-Trennzeichen behandelt,
/// sodass z.B. "Big Farm" sowohl "Big_Farm" als auch "BigFarm" matcht.
fn name_to_pattern(variant: &str) -> String {
    // Regex-Metazeichen escapen (außer Spaces/Underscores)
    let mut pattern = String::new();
    for ch in variant.chars() {
        if ch == ' ' || ch == '_' {
            pattern.push_str(".*");
        } else if ".+*?^${}()|[]\\".contains(ch) {
            pattern.push('\\');
            pattern.push(ch);
        } else {
            pattern.push(ch);
        }
    }
    pattern
}

/// Kürzt den Map-Namen auf die ersten zwei Wörter (getrennt durch `_` oder Leerzeichen).
///
/// Beispiel: `"Sickinger_Hoehe_Rheinland_Pfalz"` → `"Sickinger_Hoehe"`
fn truncate_to_two_words(name: &str) -> String {
    let parts: Vec<&str> = name.split(['_', ' ']).collect();
    let count = parts.len().min(2);
    parts[..count].join("_")
}

/// Sucht im Mods-Verzeichnis nach ZIP-Dateien, die zum Map-Namen passen.
///
/// Verwendet nur die ersten zwei Wörter des Map-Namens für die Suche.
/// Matching: case-insensitive, Spaces/Underscores als Wildcard,
/// bidirektionale Umlaut-Expansion (ä↔ae, ö↔oe, ü↔ue, ß↔ss).
fn find_matching_zips(mods_dir: &Path, map_name: &str) -> Vec<PathBuf> {
    let short_name = truncate_to_two_words(map_name);
    let variants = expand_umlaut_variants(&short_name);
    let patterns: Vec<String> = variants.iter().map(|v| name_to_pattern(v)).collect();

    // Regex-Patterns kompilieren (case-insensitive)
    let regexes: Vec<Regex> = patterns
        .iter()
        .filter_map(|p| Regex::new(&format!("(?i){}", p)).ok())
        .collect();

    if regexes.is_empty() {
        return Vec::new();
    }

    let Ok(entries) = std::fs::read_dir(mods_dir) else {
        return Vec::new();
    };

    let mut results = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(ext) = path.extension() else {
            continue;
        };
        if !ext.eq_ignore_ascii_case("zip") {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if regexes.iter().any(|re| re.is_match(file_name)) {
            results.push(path);
        }
    }

    results.sort();
    results.dedup();
    results
}

#[cfg(test)]
mod tests;
