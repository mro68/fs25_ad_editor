//! Zentrale Konfiguration für den FS25 AutoDrive Editor.
//!
//! `EditorOptions` enthält alle zur Laufzeit änderbaren Werte.
//! Die `const`-Werte bleiben als Fallback/Default erhalten.

use serde::{Deserialize, Serialize};

// ── Kamera ──────────────────────────────────────────────────────────

/// Sichtbare Welt-Halbbreite bei Zoom 1.0 (Einheiten = AutoDrive-Meter).
pub const CAMERA_BASE_WORLD_EXTENT: f32 = 2048.0;
/// Minimaler Zoom-Faktor.
pub const CAMERA_ZOOM_MIN: f32 = 0.1;
/// Maximaler Zoom-Faktor.
pub const CAMERA_ZOOM_MAX: f32 = 100.0;
/// Zoom-Schritt bei stufenweisem Zoom (Menü-Buttons / Shortcuts).
pub const CAMERA_ZOOM_STEP: f32 = 1.2;
/// Zoom-Schritt bei Mausrad-Scroll.
pub const CAMERA_SCROLL_ZOOM_STEP: f32 = 1.1;

// ── Tools ───────────────────────────────────────────────────────────

/// Snap-Radius (Welteinheiten): Klick innerhalb dieses Radius rastet auf existierenden Node ein.
pub const SNAP_RADIUS: f32 = 3.0;
/// Standard-Hitbox-Skalierung in Prozent der Node-Größe.
pub const HITBOX_SCALE_PERCENT: f32 = 100.0;

// ── Terrain ─────────────────────────────────────────────────────────

/// Standard-Terrain-Höhenskala (FS25: normalized_pixel × Faktor = Y-Meter).
pub const TERRAIN_HEIGHT_SCALE: f32 = 255.0;

// ── Selektion ───────────────────────────────────────────────────────

/// Pick-Radius in Screen-Pixeln.
pub const SELECTION_PICK_RADIUS_PX: f32 = 12.0;
/// Größenfaktor für selektierte Nodes.
pub const SELECTION_SIZE_FACTOR: f32 = 1.8;

// ── Node-Rendering ─────────────────────────────────────────────────

/// Standard-Node-Größe in Welteinheiten.
pub const NODE_SIZE_WORLD: f32 = 0.5;
/// Standard-Farbe normaler Nodes (RGBA: Cyan).
pub const NODE_COLOR_DEFAULT: [f32; 4] = [0.0, 0.8, 1.0, 1.0];
/// Farbe für Sub-Prioritäts-Nodes (RGBA: Gelb).
pub const NODE_COLOR_SUBPRIO: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
/// Farbe für selektierte Nodes (RGBA: Magenta).
pub const NODE_COLOR_SELECTED: [f32; 4] = [1.0, 0.0, 1.0, 1.0];
/// Farbe für Nodes mit Warnungen (RGBA: Rot).
pub const NODE_COLOR_WARNING: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

// ── Connection-Rendering ───────────────────────────────────────────

/// Linienstärke normaler Verbindungen in Welteinheiten.
pub const CONNECTION_THICKNESS_WORLD: f32 = 0.2;
/// Linienstärke für Sub-Prioritäts-Verbindungen.
pub const CONNECTION_THICKNESS_SUBPRIO_WORLD: f32 = 0.1;
/// Pfeil-Länge in Welteinheiten.
pub const ARROW_LENGTH_WORLD: f32 = 1.0;
/// Pfeil-Breite in Welteinheiten.
pub const ARROW_WIDTH_WORLD: f32 = 0.6;
/// Farbe für reguläre (Einrichtungs-)Verbindungen (RGBA: Grün).
pub const CONNECTION_COLOR_REGULAR: [f32; 4] = [0.2, 0.9, 0.2, 1.0];
/// Farbe für bidirektionale (Dual-)Verbindungen (RGBA: Blau).
pub const CONNECTION_COLOR_DUAL: [f32; 4] = [0.2, 0.7, 1.0, 1.0];
/// Farbe für Rückwärts-Verbindungen (RGBA: Orange).
pub const CONNECTION_COLOR_REVERSE: [f32; 4] = [1.0, 0.5, 0.1, 1.0];

// ── Map-Marker-Rendering ───────────────────────────────────────────

/// Marker-Größe in Welteinheiten.
pub const MARKER_SIZE_WORLD: f32 = 2.0;
/// Füllfarbe der Map-Marker (RGBA: Rot).
pub const MARKER_COLOR: [f32; 4] = [0.9, 0.1, 0.1, 1.0];
/// Outline-Farbe der Map-Marker (RGBA: Dunkelrot).
pub const MARKER_OUTLINE_COLOR: [f32; 4] = [0.6, 0.0, 0.0, 1.0];

// ── Übersichtskarten-Layer ──────────────────────────────────────────

/// Konfigurierbare Layer-Optionen für die Übersichtskarten-Generierung.
/// Wird als Teil der `EditorOptions` persistent gespeichert.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OverviewLayerOptions {
    /// Hillshade-Schattierung anwenden
    pub hillshade: bool,
    /// Farmland-Grenzen einzeichnen
    pub farmlands: bool,
    /// Farmland-ID-Nummern einzeichnen
    pub farmland_ids: bool,
    /// POI-Marker einzeichnen
    pub pois: bool,
    /// Legende einzeichnen
    pub legend: bool,
}

impl Default for OverviewLayerOptions {
    fn default() -> Self {
        Self {
            hillshade: true,
            farmlands: true,
            farmland_ids: true,
            pois: true,
            legend: false,
        }
    }
}

// ── Laufzeit-Optionen (serialisierbar) ─────────────────────────────

/// Alle zur Laufzeit änderbaren Editor-Optionen.
/// Wird als `fs25_auto_drive_editor.toml` neben der Binary gespeichert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorOptions {
    // ── Nodes ───────────────────────────────────────────────────
    /// Node-Größe in Welteinheiten
    pub node_size_world: f32,
    /// Standard-Farbe normaler Nodes (RGBA)
    pub node_color_default: [f32; 4],
    /// Farbe für Sub-Prioritäts-Nodes
    pub node_color_subprio: [f32; 4],
    /// Farbe für selektierte Nodes
    pub node_color_selected: [f32; 4],
    /// Farbe für Nodes mit Warnung
    pub node_color_warning: [f32; 4],

    // ── Selektion ───────────────────────────────────────────────
    /// Vergrößerungsfaktor für selektierte Nodes (Hitbox und Darstellung)
    pub selection_size_factor: f32,
    /// Pick-Radius für Klick-Selektion in Screen-Pixeln
    pub selection_pick_radius_px: f32,

    // ── Connections ─────────────────────────────────────────────
    /// Linienstärke normaler Verbindungen in Welteinheiten
    pub connection_thickness_world: f32,
    /// Linienstärke für Sub-Prioritäts-Verbindungen
    pub connection_thickness_subprio_world: f32,
    /// Pfeil-Länge in Welteinheiten
    pub arrow_length_world: f32,
    /// Pfeil-Breite in Welteinheiten
    pub arrow_width_world: f32,
    /// Farbe für Regular-Verbindungen (Einrichtung)
    pub connection_color_regular: [f32; 4],
    /// Farbe für Dual-Verbindungen (bidirektional)
    pub connection_color_dual: [f32; 4],
    /// Farbe für Reverse-Verbindungen
    pub connection_color_reverse: [f32; 4],

    // ── Marker ──────────────────────────────────────────────────
    /// Marker-Größe in Welteinheiten
    pub marker_size_world: f32,
    /// Füllfarbe der Map-Marker
    pub marker_color: [f32; 4],
    /// Outline-Farbe der Map-Marker
    pub marker_outline_color: [f32; 4],

    // ── Kamera ──────────────────────────────────────────────────
    /// Minimaler Zoom-Faktor (konfigurierbar)
    pub camera_zoom_min: f32,
    /// Maximaler Zoom-Faktor (konfigurierbar)
    pub camera_zoom_max: f32,
    /// Zoom-Schritt bei Menü-Buttons / Shortcuts
    pub camera_zoom_step: f32,
    /// Zoom-Schritt bei Mausrad-Scroll
    pub camera_scroll_zoom_step: f32,
    /// Standard-Deckungs-Niveau des Hintergrundbilds
    pub background_opacity_default: f32,
    /// Minimales Deckungs-Niveau des Hintergrundbilds bei Minimal-Zoom
    #[serde(default = "default_background_opacity_at_min_zoom")]
    pub background_opacity_at_min_zoom: f32,

    // ── Tools ────────────────────────────────────────────────────
    /// Snap-Radius (Welteinheiten) für Route-Tools
    pub snap_radius: f32,
    /// Hitbox-Skalierung in Prozent der Node-Größe (100 = exakte Node-Größe)
    #[serde(default = "default_hitbox_scale_percent")]
    pub hitbox_scale_percent: f32,
    // ── AddNode-Verhalten ─────────────────────────────────────────────
    /// Angrenzende Nodes automatisch verbinden wenn ein Node gelöscht wird
    #[serde(default)]
    pub reconnect_on_delete: bool,
    /// Verbindung trennen und durch neuen Node führen wenn er auf einer Verbindung platziert wird
    #[serde(default)]
    pub split_connection_on_place: bool,
    // ── Terrain ──────────────────────────────────────────────────
    /// Höhenskala für Heightmap-Export (FS25: 255.0)
    pub terrain_height_scale: f32,

    // ── Übersichtskarte ─────────────────────────────────────────
    /// Layer-Optionen für Übersichtskarten-Generierung
    #[serde(default)]
    pub overview_layers: OverviewLayerOptions,
}

impl Default for EditorOptions {
    fn default() -> Self {
        Self {
            node_size_world: NODE_SIZE_WORLD,
            node_color_default: NODE_COLOR_DEFAULT,
            node_color_subprio: NODE_COLOR_SUBPRIO,
            node_color_selected: NODE_COLOR_SELECTED,
            node_color_warning: NODE_COLOR_WARNING,

            selection_size_factor: SELECTION_SIZE_FACTOR,
            selection_pick_radius_px: SELECTION_PICK_RADIUS_PX,

            connection_thickness_world: CONNECTION_THICKNESS_WORLD,
            connection_thickness_subprio_world: CONNECTION_THICKNESS_SUBPRIO_WORLD,
            arrow_length_world: ARROW_LENGTH_WORLD,
            arrow_width_world: ARROW_WIDTH_WORLD,
            connection_color_regular: CONNECTION_COLOR_REGULAR,
            connection_color_dual: CONNECTION_COLOR_DUAL,
            connection_color_reverse: CONNECTION_COLOR_REVERSE,

            marker_size_world: MARKER_SIZE_WORLD,
            marker_color: MARKER_COLOR,
            marker_outline_color: MARKER_OUTLINE_COLOR,

            camera_zoom_min: CAMERA_ZOOM_MIN,
            camera_zoom_max: CAMERA_ZOOM_MAX,
            camera_zoom_step: CAMERA_ZOOM_STEP,
            camera_scroll_zoom_step: CAMERA_SCROLL_ZOOM_STEP,
            background_opacity_default: 1.0,
            background_opacity_at_min_zoom: 0.3,

            snap_radius: SNAP_RADIUS,
            hitbox_scale_percent: HITBOX_SCALE_PERCENT,
            reconnect_on_delete: false,
            split_connection_on_place: false,
            terrain_height_scale: TERRAIN_HEIGHT_SCALE,
            overview_layers: OverviewLayerOptions::default(),
        }
    }
}

/// Serde-Default für `hitbox_scale_percent` (Abwärtskompatibilität bestehender TOML-Dateien).
fn default_hitbox_scale_percent() -> f32 {
    HITBOX_SCALE_PERCENT
}

/// Serde-Default für `background_opacity_at_min_zoom` (Abwärtskompatibilität).
fn default_background_opacity_at_min_zoom() -> f32 {
    0.3
}

impl EditorOptions {
    /// Lädt Optionen aus einer TOML-Datei. Bei Fehler: Standardwerte.
    pub fn load_from_file(path: &std::path::Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(opts) => {
                    log::info!("Optionen geladen aus: {}", path.display());
                    opts
                }
                Err(e) => {
                    log::warn!("Optionen-Datei fehlerhaft, verwende Standardwerte: {}", e);
                    Self::default()
                }
            },
            Err(_) => {
                log::info!("Keine Optionen-Datei gefunden, verwende Standardwerte");
                Self::default()
            }
        }
    }

    /// Speichert Optionen als TOML-Datei.
    pub fn save_to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        log::info!("Optionen gespeichert nach: {}", path.display());
        Ok(())
    }

    /// Ermittelt den Pfad zur Optionen-Datei neben der Binary.
    pub fn config_path() -> std::path::PathBuf {
        std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("fs25_auto_drive_editor"))
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("fs25_auto_drive_editor.toml")
    }

    /// Berechnet den Hitbox-Radius in Welteinheiten.
    ///
    /// `node_size_world * hitbox_scale_percent / 100`
    pub fn hitbox_radius(&self) -> f32 {
        self.node_size_world * self.hitbox_scale_percent / 100.0
    }
}
