//! Use-Case-Funktionen fuer Laden und Speichern der Editor-Optionen.

use crate::shared::EditorOptions;
use anyhow::Context;
use std::path::{Path, PathBuf};

const OPTIONS_FILE_NAME: &str = "fs25_auto_drive_editor.toml";

/// Laedt Editor-Optionen vom Standardpfad neben der Binary.
pub fn load_editor_options() -> EditorOptions {
    let path = config_path();
    load_editor_options_from_file(&path)
}

/// Laedt Editor-Optionen aus einer TOML-Datei.
///
/// Bei Lese-, Parse- oder Validierungsfehlern werden Standardwerte zurueckgegeben.
pub fn load_editor_options_from_file(path: &Path) -> EditorOptions {
    match std::fs::read_to_string(path) {
        Ok(content) => match toml::from_str::<EditorOptions>(&content) {
            Ok(opts) => match normalize_loaded_options(opts) {
                Ok(normalized) => {
                    log::info!("Optionen geladen aus: {}", path.display());
                    normalized
                }
                Err(error) => {
                    log::warn!(
                        "Optionen-Validierung fehlgeschlagen, verwende Standardwerte: {}",
                        error
                    );
                    EditorOptions::default()
                }
            },
            Err(error) => {
                log::warn!(
                    "Optionen-Datei fehlerhaft, verwende Standardwerte: {}",
                    error
                );
                EditorOptions::default()
            }
        },
        Err(_) => {
            log::info!("Keine Optionen-Datei gefunden, verwende Standardwerte");
            EditorOptions::default()
        }
    }
}

/// Speichert Editor-Optionen am Standardpfad neben der Binary.
pub fn save_editor_options(options: &EditorOptions) -> anyhow::Result<()> {
    let path = config_path();
    save_editor_options_to_file(&path, options)
}

/// Speichert Editor-Optionen als TOML-Datei.
pub fn save_editor_options_to_file(path: &Path, options: &EditorOptions) -> anyhow::Result<()> {
    options.validate()?;
    let content = toml::to_string_pretty(options)?;
    std::fs::write(path, content).with_context(|| {
        format!(
            "Optionen-Datei konnte nicht geschrieben werden: {}",
            path.display()
        )
    })?;
    log::info!("Optionen gespeichert nach: {}", path.display());
    Ok(())
}

/// Ermittelt den Standardpfad zur Optionen-Datei neben der Binary.
pub fn config_path() -> PathBuf {
    std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("fs25_auto_drive_editor"))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(OPTIONS_FILE_NAME)
}

fn normalize_loaded_options(mut options: EditorOptions) -> anyhow::Result<EditorOptions> {
    if options.selection_size_factor > 0.0 && options.selection_size_factor <= 5.0 {
        options.selection_size_factor *= 100.0;
    }
    options.validate()?;
    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_options_path(suffix: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Systemzeit muss nach Unix-Epoche liegen")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "fs25_ad_editor_options_{}_{}_{}.toml",
            std::process::id(),
            suffix,
            timestamp
        ))
    }

    fn cleanup_options_path(path: &Path) {
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn load_editor_options_from_file_uses_defaults_when_missing() {
        let path = temp_options_path("missing");
        let loaded = load_editor_options_from_file(&path);

        assert_eq!(
            loaded.selection_size_factor,
            EditorOptions::default().selection_size_factor
        );
    }

    #[test]
    fn load_editor_options_from_file_uses_defaults_when_toml_is_invalid() {
        let path = temp_options_path("invalid_toml");
        let defaults = EditorOptions::default();

        std::fs::write(&path, "selection_size_factor = [\n")
            .expect("Defektes TOML muss geschrieben werden");

        let loaded = load_editor_options_from_file(&path);
        cleanup_options_path(&path);

        assert_eq!(loaded.selection_size_factor, defaults.selection_size_factor);
        assert_eq!(loaded.camera_zoom_min, defaults.camera_zoom_min);
    }

    #[test]
    fn load_editor_options_from_file_uses_defaults_when_validation_fails() {
        let path = temp_options_path("invalid_values");
        let defaults = EditorOptions::default();
        let mut invalid = EditorOptions::default();
        invalid.camera_zoom_min = invalid.camera_zoom_max;

        std::fs::write(
            &path,
            toml::to_string_pretty(&invalid).expect("Invalides TOML muss serialisierbar sein"),
        )
        .expect("Invalides TOML muss geschrieben werden");

        let loaded = load_editor_options_from_file(&path);
        cleanup_options_path(&path);

        assert_eq!(loaded.camera_zoom_min, defaults.camera_zoom_min);
        assert_eq!(loaded.camera_zoom_max, defaults.camera_zoom_max);
    }

    #[test]
    fn load_editor_options_from_file_normalizes_legacy_selection_factor() {
        let path = temp_options_path("legacy");
        let mut legacy = EditorOptions::default();
        legacy.selection_size_factor = 1.4;

        std::fs::write(
            &path,
            toml::to_string_pretty(&legacy).expect("Legacy-TOML muss serialisierbar sein"),
        )
        .expect("Legacy-TOML muss geschrieben werden");

        let loaded = load_editor_options_from_file(&path);
        cleanup_options_path(&path);

        assert!((loaded.selection_size_factor - 140.0).abs() < f32::EPSILON);
    }

    #[test]
    fn save_editor_options_to_file_writes_toml() {
        let path = temp_options_path("save");

        save_editor_options_to_file(&path, &EditorOptions::default())
            .expect("Optionen muessen gespeichert werden");

        let content =
            std::fs::read_to_string(&path).expect("Gespeicherte Optionen muessen lesbar sein");
        cleanup_options_path(&path);

        assert!(
            content.contains("selection_size_factor"),
            "TOML muss bekannte Optionsfelder enthalten"
        );
    }

    #[test]
    fn save_editor_options_to_file_rejects_invalid_options_before_write() {
        let path = temp_options_path("save_invalid");
        let mut invalid = EditorOptions::default();
        invalid.camera_zoom_min = invalid.camera_zoom_max;

        let error = save_editor_options_to_file(&path, &invalid)
            .expect_err("Ungueltige Optionen duerfen nicht gespeichert werden");

        assert!(
            error.to_string().contains("camera_zoom_min"),
            "Fehlermeldung soll die Validierungsursache enthalten"
        );
        assert!(
            !path.exists(),
            "Bei fehlgeschlagener Validierung darf keine Datei geschrieben werden"
        );
    }
}
