/// Repräsentiert einen Map-Marker aus der AutoDrive-Config
/// Ein Map-Marker verweist auf einen Node und wird im Editor angezeigt.
/// Beschreibt einen Map-Marker in der AutoDrive-Konfiguration.
#[derive(Debug, Clone)]
pub struct MapMarker {
    /// Node-ID des Markers
    pub id: u64,
    /// Anzeigename
    pub name: String,
    /// Marker-Gruppe
    pub group: String,
    /// Laufende Nummer im XML
    pub marker_index: u32,
    /// Debug-Marker (nur intern)
    pub is_debug: bool,
}

impl MapMarker {
    /// Erstellt einen neuen Map-Marker
    pub fn new(id: u64, name: String, group: String, marker_index: u32, is_debug: bool) -> Self {
        Self {
            id,
            name,
            group,
            marker_index,
            is_debug,
        }
    }
}
