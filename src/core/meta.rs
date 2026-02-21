//! Nicht-renderrelevante Metadaten aus der AutoDrive-XML-Konfiguration.

/// Container für zusätzliche AutoDrive-Metadaten (Versions-Strings, Optionen).
#[derive(Debug, Clone, Default)]
pub struct AutoDriveMeta {
    /// Exakte Versionszeichenkette aus der XML (z.B. 3.0.0.4)
    pub config_version: Option<String>,
    /// Version der gespeicherten Route
    pub route_version: Option<String>,
    /// Autor der Route
    pub route_author: Option<String>,
    /// Sonstige Optionen aus der Config (in Original-Reihenfolge)
    pub options: Vec<(String, String)>,
}
