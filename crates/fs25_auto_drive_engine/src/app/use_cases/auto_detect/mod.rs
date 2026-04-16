//! Auto-Detection von Heightmap und Map-Mod-ZIP nach dem Laden einer XML-Datei.
//!
//! Prueft nach dem Laden einer AutoDrive-Config, ob:
//! 1. Eine `terrain.heightmap.png` im selben Verzeichnis liegt → direkt als Heightmap setzen
//! 2. Ein gespeichertes Overview-Layer-Bundle im XML-Verzeichnis liegt → spaeter bevorzugt laden
//! 3. Im XML-Verzeichnis oder im Mods-Verzeichnis ein passendes ZIP zum `map_name` existiert
//!    → Dialog anzeigen

use crate::app::use_cases::background_layers::discover_background_layer_files;
use crate::app::BackgroundLayerFiles;
use regex::Regex;
use std::path::{Path, PathBuf};

/// Ergebnis der Auto-Detection nach dem Laden einer XML-Datei.
#[derive(Debug, Default)]
pub struct PostLoadDetectionResult {
    /// Pfad zur gefundenen Heightmap (falls vorhanden)
    pub heightmap_path: Option<PathBuf>,
    /// Gefundene gespeicherte Layer-Dateien mit Terrain-Basis im XML-Verzeichnis.
    pub background_layer_files: Option<BackgroundLayerFiles>,
    /// Pfad zu einer gefundenen overview.png im XML-Verzeichnis
    pub overview_path: Option<PathBuf>,
    /// Passende ZIP-Dateien aus XML-Verzeichnis und Mods-Verzeichnis
    pub matching_zips: Vec<PathBuf>,
}

/// Fuehrt die komplette Auto-Detection durch.
///
/// Sucht nach `terrain.heightmap.png`, nach einem gespeicherten Overview-Layer-Bundle
/// (Terrain-Basis plus optionale Overlays) im XML-Verzeichnis und nach passenden
/// Map-Mod-ZIPs zuerst im XML-Verzeichnis und zusaetzlich im Mods-Verzeichnis
/// (basierend auf `map_name`). Ohne `overview_terrain.png` bleibt das Layer-System
/// fuer diesen Load inaktiv und der Legacy-Fallback ueber `overview.png`/`.jpg` aktiv.
pub fn detect_post_load(xml_path: &Path, map_name: Option<&str>) -> PostLoadDetectionResult {
    let heightmap_path = find_heightmap_next_to(xml_path);
    let background_layer_files = xml_path.parent().and_then(detect_background_layer_files);
    let overview_path = find_overview_next_to(xml_path);
    let matching_zips = map_name
        .filter(|name| !name.is_empty())
        .map_or_else(Vec::new, |name| {
            let mut results = Vec::new();

            if let Some(xml_dir) = xml_path.parent() {
                extend_unique_paths(&mut results, find_matching_zips(xml_dir, name));
            }

            if let Some(mods_dir) = resolve_mods_dir(xml_path) {
                extend_unique_paths(&mut results, find_matching_zips(&mods_dir, name));
            }

            results
        });

    PostLoadDetectionResult {
        heightmap_path,
        background_layer_files,
        overview_path,
        matching_zips,
    }
}

fn detect_background_layer_files(dir: &Path) -> Option<BackgroundLayerFiles> {
    let files = discover_background_layer_files(dir);
    files.terrain.is_some().then_some(files)
}

fn extend_unique_paths(results: &mut Vec<PathBuf>, candidates: Vec<PathBuf>) {
    for candidate in candidates {
        if !results.contains(&candidate) {
            results.push(candidate);
        }
    }
}

/// Prueft ob `overview.png` (oder `overview.jpg`) im selben Verzeichnis wie die XML liegt.
fn find_overview_next_to(xml_path: &Path) -> Option<PathBuf> {
    let dir = xml_path.parent()?;
    // Bevorzugt .png (verlustfrei), Fallback auf .jpg (Abwaertskompatibilitaet)
    for name in &["overview.png", "overview.jpg"] {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Prueft ob `terrain.heightmap.png` im selben Verzeichnis wie die XML liegt.
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
/// Ersetzt (a-umlaut)<->ae, (o-umlaut)<->oe, (u-umlaut)<->ue, (eszett)<->ss in beide Richtungen.
/// Gibt alle eindeutigen Varianten zurueck (inkl. Original).
fn expand_umlaut_variants(name: &str) -> Vec<String> {
    let lower = name.to_lowercase();

    // Richtung 1: Umlaute → ASCII-Digraphen
    let ascii = lower
        .replace('\u{00E4}', "ae")
        .replace('\u{00F6}', "oe")
        .replace('\u{00FC}', "ue")
        .replace('\u{00DF}', "ss");

    // Richtung 2: ASCII-Digraphen → Umlaute
    let umlaut = lower
        .replace("ae", "\u{00E4}")
        .replace("oe", "\u{00F6}")
        .replace("ue", "\u{00FC}")
        .replace("ss", "\u{00DF}");

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
    // Regex-Metazeichen escapen (ausser Spaces/Underscores)
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

/// Kuerzt den Map-Namen auf die ersten zwei Woerter (getrennt durch `_` oder Leerzeichen).
///
/// Beispiel: `"Sickinger_Hoehe_Rheinland_Pfalz"` → `"Sickinger_Hoehe"`
fn truncate_to_two_words(name: &str) -> String {
    let parts: Vec<&str> = name.split(['_', ' ']).collect();
    let count = parts.len().min(2);
    parts[..count].join("_")
}

/// Sucht in einem Verzeichnis nach ZIP-Dateien, die zum Map-Namen passen.
///
/// Verwendet nur die ersten zwei Woerter des Map-Namens fuer die Suche.
/// Matching: case-insensitive, Spaces/Underscores als Wildcard,
/// bidirektionale Umlaut-Expansion (ae↔ae, oe↔oe, ue↔ue, ss↔ss).
fn find_matching_zips(search_dir: &Path, map_name: &str) -> Vec<PathBuf> {
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

    let Ok(entries) = std::fs::read_dir(search_dir) else {
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
