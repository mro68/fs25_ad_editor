//! Dialog-Anforderungs- und -Ergebnis-DTOs fuer die Host-Bridge.

use serde::{Deserialize, Serialize};

/// Stabile Art eines Host-Datei-/Pfad-Dialogs fuer die Bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostDialogRequestKind {
    /// AutoDrive-XML laden.
    OpenFile,
    /// AutoDrive-XML speichern.
    SaveFile,
    /// Heightmap-Bild auswaehlen.
    Heightmap,
    /// Hintergrundbild oder ZIP auswaehlen.
    BackgroundMap,
    /// Map-Mod-ZIP fuer Overview-Generierung auswaehlen.
    OverviewZip,
    /// Curseplay-Datei importieren.
    CurseplayImport,
    /// Curseplay-Datei exportieren.
    CurseplayExport,
}

/// Serialisierbare Dialog-Anforderung fuer Hosts ohne direkten Engine-State-Zugriff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostDialogRequest {
    /// Semantische Bedeutung der Anfrage.
    pub kind: HostDialogRequestKind,
    /// Optionaler Dateiname fuer Save-Dialoge.
    pub suggested_file_name: Option<String>,
}

/// Serialisierbare Rueckmeldung eines Hosts zu einer Dialog-Anforderung.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum HostDialogResult {
    /// Host-Dialog wurde ohne Auswahl geschlossen.
    Cancelled {
        /// Semantische Art der beantworteten Anfrage.
        kind: HostDialogRequestKind,
    },
    /// Host hat einen Pfad ausgewaehlt.
    PathSelected {
        /// Semantische Art der beantworteten Anfrage.
        kind: HostDialogRequestKind,
        /// Gewaehlter Pfad.
        path: String,
    },
}
