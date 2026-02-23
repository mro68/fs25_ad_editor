//! Map-Discovery: Erkennung der Kartenstruktur in FS25-Map-Mod-ZIPs.
//!
//! Parst `modDesc.xml` und die Map-Config-XML um Kartengröße,
//! Datenverzeichnisse und relevante Dateipfade zu ermitteln.

use anyhow::{bail, Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Erkannte Kartenstruktur aus einem Map-Mod.
#[derive(Debug, Clone)]
pub struct MapInfo {
    /// Kartentitel (aus modDesc.xml)
    pub title: String,
    /// Kartengröße in Pixeln (quadratisch)
    pub map_size: u32,
    /// Pfad zur Map-Config-XML relativ zum Mod-Root
    pub config_path: String,
    /// Pfad zum data/-Verzeichnis relativ zum Mod-Root
    pub data_dir: String,
    /// Pfad zum config/-Verzeichnis relativ zum Mod-Root
    pub config_dir: String,
    /// Pfad zur placeables.xml relativ zum Mod-Root (optional)
    pub placeables_path: Option<String>,
}

/// Erkennt die Kartenstruktur aus den Dateien eines Map-Mod-ZIPs.
///
/// # Parameter
/// - `files`: HashMap mit Dateipfaden (relativ zum ZIP-Root) → Dateiinhalte
///
/// # Ablauf
/// 1. `modDesc.xml` finden und parsen
/// 2. Map-Config-XML lokalisieren
/// 3. Kartengröße aus Config-XML lesen
/// 4. `data/`- und `config/`-Verzeichnisse bestimmen
pub fn discover_map(files: &HashMap<String, Vec<u8>>) -> Result<MapInfo> {
    // modDesc.xml finden (kann in Root oder einem Unterverzeichnis liegen)
    let (mod_desc_path, mod_desc_content) = find_file(files, "modDesc.xml")
        .context("modDesc.xml nicht im ZIP gefunden – ist das ein FS25-Map-Mod?")?;

    let mod_root = parent_dir(&mod_desc_path);

    // modDesc.xml parsen
    let (title, config_filename, placeables_filename) = parse_mod_desc(mod_desc_content)?;

    // Map-Config-XML finden und parsen
    let config_path = join_paths(&mod_root, &config_filename);
    let config_content = files
        .get(&config_path)
        .with_context(|| format!("Map-Config-XML nicht gefunden: {}", config_path))?;

    let map_size = parse_map_config(config_content)?;

    // data/-Verzeichnis bestimmen
    let config_dir_path = parent_dir(&config_path);
    let data_dir = find_data_dir(files, &config_dir_path, &mod_root);

    // config/-Verzeichnis (Sibling von data/ oder neben der Config-XML)
    let map_dir = parent_dir(&data_dir);
    let config_dir = format!("{}/config", map_dir.trim_end_matches('/'));

    // Placeables-Pfad
    let placeables_path = placeables_filename.map(|f| join_paths(&mod_root, &f));

    log::info!(
        "Map erkannt: '{}', {}x{}, data='{}'",
        title,
        map_size,
        map_size,
        data_dir
    );

    Ok(MapInfo {
        title,
        map_size,
        config_path,
        data_dir,
        config_dir,
        placeables_path,
    })
}

/// Parst modDesc.xml: Titel, configFilename, placeablesFilename.
fn parse_mod_desc(content: &[u8]) -> Result<(String, String, Option<String>)> {
    let mut reader = Reader::from_reader(content);
    reader.config_mut().trim_text(true);

    let mut title = String::from("FS25 Map");
    let mut config_filename = String::new();
    let mut placeables_filename = None;

    let mut in_title = false;
    let mut in_title_en = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "title" {
                    in_title = true;
                } else if in_title && name == "en" {
                    in_title_en = true;
                } else if name == "map" {
                    // <map configFilename="..." defaultPlaceablesXMLFilename="...">
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        if key == "configFilename" {
                            config_filename = val;
                        } else if key == "defaultPlaceablesXMLFilename" {
                            placeables_filename = Some(val);
                        }
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "map" {
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        if key == "configFilename" {
                            config_filename = val;
                        } else if key == "defaultPlaceablesXMLFilename" {
                            placeables_filename = Some(val);
                        }
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if in_title_en {
                    title = String::from_utf8_lossy(e.as_ref()).to_string();
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "title" {
                    in_title = false;
                } else if name == "en" {
                    in_title_en = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => bail!("Fehler beim Parsen von modDesc.xml: {}", e),
            _ => {}
        }
        buf.clear();
    }

    if config_filename.is_empty() {
        bail!("Kein <map configFilename> in modDesc.xml gefunden");
    }

    Ok((title, config_filename, placeables_filename))
}

/// Parst die Map-Config-XML für width/height.
fn parse_map_config(content: &[u8]) -> Result<u32> {
    let mut reader = Reader::from_reader(content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e) | Event::Empty(e)) => {
                // Das Root-Element hat width/height Attribute
                let mut width = None;
                let mut height = None;
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let val = String::from_utf8_lossy(&attr.value).to_string();
                    if key == "width" {
                        width = val.parse::<u32>().ok();
                    } else if key == "height" {
                        height = val.parse::<u32>().ok();
                    }
                }
                if let (Some(w), Some(h)) = (width, height) {
                    return Ok(w.max(h));
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => bail!("Fehler beim Parsen der Map-Config-XML: {}", e),
            _ => {}
        }
        buf.clear();
    }

    // Fallback: Standardgröße
    log::warn!("Keine width/height in Map-Config gefunden, verwende Standard 2048");
    Ok(2048)
}

/// Sucht das data/-Verzeichnis anhand bekannter Dateien.
fn find_data_dir(files: &HashMap<String, Vec<u8>>, config_dir: &str, mod_root: &str) -> String {
    // Versuch 1: Sibling des Config-Dir
    let candidate1 = format!("{}/data", config_dir.trim_end_matches('/'));
    if has_dir_prefix(files, &candidate1) {
        return candidate1;
    }

    // Versuch 2: Parent des Config-Dir + /data
    let parent = parent_dir(config_dir);
    let candidate2 = format!("{}/data", parent.trim_end_matches('/'));
    if has_dir_prefix(files, &candidate2) {
        return candidate2;
    }

    // Versuch 3: Suche nach Dateien wie dem.png oder *_weight.png
    for path in files.keys() {
        let lower = path.to_ascii_lowercase();
        if lower.ends_with("/dem.png") || lower.ends_with("_weight.png") {
            if let Some(dir) = Path::new(path).parent() {
                return dir.to_string_lossy().to_string();
            }
        }
    }

    // Fallback: mod_root/data
    format!("{}/data", mod_root.trim_end_matches('/'))
}

/// Findet eine Datei im ZIP (case-insensitive für den Basename).
fn find_file<'a>(
    files: &'a HashMap<String, Vec<u8>>,
    target_basename: &str,
) -> Option<(String, &'a [u8])> {
    let target_lower = target_basename.to_ascii_lowercase();
    for (path, content) in files {
        if let Some(name) = Path::new(path).file_name() {
            if name.to_string_lossy().to_ascii_lowercase() == target_lower {
                return Some((path.clone(), content.as_slice()));
            }
        }
    }
    None
}

/// Überprüft ob Dateien mit einem bestimmten Verzeichnis-Prefix existieren.
fn has_dir_prefix(files: &HashMap<String, Vec<u8>>, prefix: &str) -> bool {
    let prefix_with_slash = if prefix.ends_with('/') {
        prefix.to_string()
    } else {
        format!("{}/", prefix)
    };
    files.keys().any(|k| k.starts_with(&prefix_with_slash))
}

/// Gibt das übergeordnete Verzeichnis zurück.
fn parent_dir(path: &str) -> String {
    Path::new(path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Verbindet zwei Pfadkomponenten.
fn join_paths(base: &str, relative: &str) -> String {
    if base.is_empty() {
        relative.to_string()
    } else {
        let base_path = PathBuf::from(base);
        base_path.join(relative).to_string_lossy().to_string()
    }
}

/// Listet alle Weight-Map-Dateien im data/-Verzeichnis.
pub fn find_weight_maps<'a>(
    files: &'a HashMap<String, Vec<u8>>,
    data_dir: &str,
) -> Vec<(&'a str, &'a [u8])> {
    let prefix = if data_dir.ends_with('/') {
        data_dir.to_string()
    } else {
        format!("{}/", data_dir)
    };

    let mut weight_maps: Vec<(&str, &[u8])> = files
        .iter()
        .filter(|(path, _)| path.starts_with(&prefix) && path.ends_with("_weight.png"))
        .map(|(path, content)| (path.as_str(), content.as_slice()))
        .collect();

    weight_maps.sort_by_key(|(path, _)| *path);
    weight_maps
}

/// Findet das DEM (Digital Elevation Model) im data/-Verzeichnis.
pub fn find_dem<'a>(files: &'a HashMap<String, Vec<u8>>, data_dir: &str) -> Option<&'a [u8]> {
    let path = format!("{}/dem.png", data_dir.trim_end_matches('/'));
    files.get(&path).map(|v| v.as_slice())
}

/// Findet die Farmlands-InfoLayer-Datei.
///
/// Sucht nach `infoLayer_farmlands.grle` oder `.png`.
pub fn find_farmlands<'a>(
    files: &'a HashMap<String, Vec<u8>>,
    data_dir: &str,
) -> Option<(&'a str, &'a [u8])> {
    let prefix = data_dir.trim_end_matches('/');

    // Reihenfolge: .grle (muss dekodiert werden), dann .png
    let grle_key = format!("{}/infoLayer_farmlands.grle", prefix);
    let png_key = format!("{}/infoLayer_farmlands.png", prefix);

    for key in [&grle_key, &png_key] {
        if let Some((k, v)) = files.get_key_value(key.as_str()) {
            return Some((k.as_str(), v.as_slice()));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parent_dir() {
        assert_eq!(parent_dir("maps/data/foo.png"), "maps/data");
        assert_eq!(parent_dir("foo.xml"), "");
    }

    #[test]
    fn test_join_paths() {
        assert_eq!(join_paths("maps", "config/map.xml"), "maps/config/map.xml");
        assert_eq!(join_paths("", "modDesc.xml"), "modDesc.xml");
    }
}
