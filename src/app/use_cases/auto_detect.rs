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
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_expand_umlaut_variants_with_umlauts() {
        let variants = expand_umlaut_variants("Höflingen");
        assert!(variants.contains(&"höflingen".to_string()));
        assert!(variants.contains(&"hoeflingen".to_string()));
    }

    #[test]
    fn test_expand_umlaut_variants_with_ascii() {
        let variants = expand_umlaut_variants("Hoeflingen");
        assert!(variants.contains(&"hoeflingen".to_string()));
        assert!(variants.contains(&"höflingen".to_string()));
    }

    #[test]
    fn test_expand_umlaut_variants_no_umlauts() {
        let variants = expand_umlaut_variants("Farm");
        assert_eq!(variants.len(), 1);
        assert!(variants.contains(&"farm".to_string()));
    }

    #[test]
    fn test_name_to_pattern_spaces() {
        let pattern = name_to_pattern("big farm");
        assert_eq!(pattern, "big.*farm");
    }

    #[test]
    fn test_name_to_pattern_underscores() {
        let pattern = name_to_pattern("big_farm");
        assert_eq!(pattern, "big.*farm");
    }

    #[test]
    fn test_truncate_to_two_words() {
        assert_eq!(
            truncate_to_two_words("Sickinger_Hoehe_Rheinland_Pfalz"),
            "Sickinger_Hoehe"
        );
        assert_eq!(truncate_to_two_words("Big Farm West"), "Big_Farm");
        assert_eq!(truncate_to_two_words("SingleWord"), "SingleWord");
        assert_eq!(truncate_to_two_words("Two_Words"), "Two_Words");
    }

    #[test]
    fn test_find_matching_zips() {
        let tmp = std::env::temp_dir().join("test_auto_detect_zips");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        // Testdateien anlegen
        fs::write(tmp.join("FS25_Hoeflingen.zip"), b"").unwrap();
        fs::write(tmp.join("FS25_Höflingen_V2.zip"), b"").unwrap();
        fs::write(tmp.join("FS25_Big_Farm.zip"), b"").unwrap();
        fs::write(tmp.join("FS25_Unrelated.zip"), b"").unwrap();
        fs::write(tmp.join("readme.txt"), b"").unwrap();
        fs::write(tmp.join("FS25_Sickinger_Hoehe_v3.zip"), b"").unwrap();

        // Test: "Höflingen" soll beide Höflingen-ZIPs finden
        let results = find_matching_zips(&tmp, "Höflingen");
        assert!(
            results
                .iter()
                .any(|p| p.file_name().unwrap().to_str().unwrap() == "FS25_Hoeflingen.zip"),
            "Soll FS25_Hoeflingen.zip finden, got: {:?}",
            results
        );
        assert!(
            results
                .iter()
                .any(|p| p.file_name().unwrap().to_str().unwrap() == "FS25_Höflingen_V2.zip"),
            "Soll FS25_Höflingen_V2.zip finden, got: {:?}",
            results
        );
        assert!(
            !results
                .iter()
                .any(|p| p.file_name().unwrap().to_str().unwrap() == "FS25_Unrelated.zip"),
            "Soll FS25_Unrelated.zip NICHT finden"
        );

        // Test: "Big Farm" soll FS25_Big_Farm.zip finden (Space → Underscore)
        let results2 = find_matching_zips(&tmp, "Big Farm");
        assert!(
            results2
                .iter()
                .any(|p| p.file_name().unwrap().to_str().unwrap() == "FS25_Big_Farm.zip"),
            "Soll FS25_Big_Farm.zip finden, got: {:?}",
            results2
        );

        // Test: Langer Name → nur erste 2 Wörter für Suche
        let results3 = find_matching_zips(&tmp, "Sickinger_Hoehe_Rheinland_Pfalz");
        assert!(
            results3
                .iter()
                .any(|p| p.file_name().unwrap().to_str().unwrap() == "FS25_Sickinger_Hoehe_v3.zip"),
            "Soll FS25_Sickinger_Hoehe_v3.zip finden, got: {:?}",
            results3
        );

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_find_heightmap_next_to() {
        let tmp = std::env::temp_dir().join("test_auto_detect_hm");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let xml_path = tmp.join("AutoDrive_config.xml");
        fs::write(&xml_path, b"<xml/>").unwrap();

        // Kein Heightmap → None
        assert!(find_heightmap_next_to(&xml_path).is_none());

        // Heightmap erstellen → Some
        let hm_path = tmp.join("terrain.heightmap.png");
        fs::write(&hm_path, b"PNG").unwrap();
        assert_eq!(find_heightmap_next_to(&xml_path), Some(hm_path));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_detect_post_load_integration() {
        let tmp = std::env::temp_dir().join("test_auto_detect_full");
        let _ = fs::remove_dir_all(&tmp);
        let savegame = tmp.join("savegame1");
        let mods = tmp.join("mods");
        fs::create_dir_all(&savegame).unwrap();
        fs::create_dir_all(&mods).unwrap();

        let xml_path = savegame.join("AutoDrive_config.xml");
        fs::write(&xml_path, b"<xml/>").unwrap();
        fs::write(savegame.join("terrain.heightmap.png"), b"PNG").unwrap();
        fs::write(mods.join("FS25_TestMap.zip"), b"").unwrap();

        let result = detect_post_load(&xml_path, Some("TestMap"));
        assert!(result.heightmap_path.is_some());
        assert_eq!(result.matching_zips.len(), 1);

        let _ = fs::remove_dir_all(&tmp);
    }
}
