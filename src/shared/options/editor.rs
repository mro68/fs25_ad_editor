//! Laufzeitoptionen (`EditorOptions`) inkl. Persistenz und Validierung.

use super::camera::{CAMERA_SCROLL_ZOOM_STEP, CAMERA_ZOOM_MAX, CAMERA_ZOOM_MIN, CAMERA_ZOOM_STEP};
use super::render::{
    OverviewLayerOptions, SelectionStyle, ARROW_LENGTH_WORLD, ARROW_WIDTH_WORLD,
    CONNECTION_COLOR_DUAL, CONNECTION_COLOR_REGULAR, CONNECTION_COLOR_REVERSE,
    CONNECTION_THICKNESS_SUBPRIO_WORLD, CONNECTION_THICKNESS_WORLD, DEFAULT_ZOOM_COMPENSATION_MAX,
    MARKER_COLOR, MARKER_OUTLINE_COLOR, MARKER_OUTLINE_WIDTH, MARKER_SIZE_WORLD, MIN_ARROW_SIZE_PX,
    MIN_CONNECTION_WIDTH_PX, MIN_MARKER_SIZE_PX, MIN_NODE_SIZE_PX, NODE_COLOR_DEFAULT,
    NODE_COLOR_SELECTED, NODE_COLOR_SUBPRIO, NODE_COLOR_WARNING, NODE_DECIMATION_SPACING_PX,
    NODE_SIZE_WORLD, SELECTION_SIZE_FACTOR, TERRAIN_HEIGHT_SCALE,
};
use super::tools::{
    ValueAdjustInputMode, HITBOX_SCALE_PERCENT, MOUSE_WHEEL_DISTANCE_STEP_M, SNAP_SCALE_PERCENT,
};
use serde::{Deserialize, Serialize};

/// Alle zur Laufzeit aenderbaren Editor-Optionen.
/// Wird als `fs25_auto_drive_editor.toml` neben der Binary gespeichert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorOptions {
    // Nodes
    pub node_size_world: f32,
    pub node_color_default: [f32; 4],
    pub node_color_subprio: [f32; 4],
    pub node_color_selected: [f32; 4],
    pub node_color_warning: [f32; 4],

    // Selektion
    pub selection_size_factor: f32,
    #[serde(default)]
    pub selection_style: SelectionStyle,
    /// Doppelklick-Segment: Bei Kreuzung (degree != 2) stoppen.
    #[serde(default = "default_segment_stop_at_junction")]
    pub segment_stop_at_junction: bool,
    /// Doppelklick-Segment: Max. Winkelabweichung in Grad (0 = nicht pruefen).
    #[serde(default = "default_segment_max_angle_deg")]
    pub segment_max_angle_deg: f32,

    // Connections
    pub connection_thickness_world: f32,
    pub connection_thickness_subprio_world: f32,
    pub arrow_length_world: f32,
    pub arrow_width_world: f32,
    pub connection_color_regular: [f32; 4],
    pub connection_color_dual: [f32; 4],
    pub connection_color_reverse: [f32; 4],

    // Marker
    pub marker_size_world: f32,
    pub marker_color: [f32; 4],
    pub marker_outline_color: [f32; 4],
    /// Umrissstärke des Map-Markers als Anteil am Radius (0.01–0.3).
    #[serde(default = "default_marker_outline_width")]
    pub marker_outline_width: f32,

    // Kamera
    #[serde(default = "default_camera_zoom_min")]
    pub camera_zoom_min: f32,
    #[serde(default = "default_camera_zoom_max")]
    pub camera_zoom_max: f32,
    pub camera_zoom_step: f32,
    pub camera_scroll_zoom_step: f32,

    // Tools
    #[serde(default = "default_snap_scale_percent")]
    pub snap_scale_percent: f32,
    #[serde(default = "default_hitbox_scale_percent")]
    pub hitbox_scale_percent: f32,
    #[serde(default = "default_mouse_wheel_distance_step_m")]
    pub mouse_wheel_distance_step_m: f32,
    #[serde(default)]
    pub value_adjust_input_mode: ValueAdjustInputMode,
    #[serde(default)]
    pub reconnect_on_delete: bool,
    #[serde(default)]
    pub split_connection_on_place: bool,
    /// Ob Route-Tool-Ergebnisse automatisch als Segment registriert werden.
    #[serde(default)]
    pub auto_create_segment: bool,

    // Terrain
    pub terrain_height_scale: f32,

    // Hintergrund
    #[serde(default = "default_bg_opacity")]
    pub bg_opacity: f32,
    #[serde(default = "default_bg_opacity_at_min_zoom")]
    pub bg_opacity_at_min_zoom: f32,
    #[serde(default = "default_bg_fade_start_zoom")]
    pub bg_fade_start_zoom: f32,

    // Copy/Paste
    #[serde(default = "default_copy_preview_opacity")]
    pub copy_preview_opacity: f32,

    // Segment-Overlay
    /// Schriftgroesse des Lock-Icons im Segment-Overlay in Pixeln.
    #[serde(default = "default_segment_lock_icon_size_px")]
    pub segment_lock_icon_size_px: f32,

    // Uebersichtskarte
    #[serde(default)]
    pub overview_layers: OverviewLayerOptions,

    // Zoom-Kompensation
    /// Maximaler Zoom-Kompensationsfaktor (1.0 = deaktiviert, 4.0 = Standard).
    /// Verhindert, dass Nodes und Verbindungen beim Herauszoomen unsichtbar werden.
    #[serde(default = "default_zoom_compensation_max")]
    pub zoom_compensation_max: f32,

    // LOD / Mindestgroessen
    /// Mindestgroesse fuer Nodes in Pixeln beim Zoomout (0.0 = deaktiviert).
    #[serde(default = "default_min_node_size_px")]
    pub min_node_size_px: f32,
    /// Mindestbreite fuer Verbindungslinien in Pixeln beim Zoomout (0.0 = deaktiviert).
    #[serde(default = "default_min_connection_width_px")]
    pub min_connection_width_px: f32,
    /// Mindestgroesse fuer Richtungspfeile in Pixeln beim Zoomout (0.0 = deaktiviert).
    #[serde(default = "default_min_arrow_size_px")]
    pub min_arrow_size_px: f32,
    /// Mindestgroesse fuer Marker-Pins in Pixeln beim Zoomout (0.0 = deaktiviert).
    #[serde(default = "default_min_marker_size_px")]
    pub min_marker_size_px: f32,
    /// Mindestabstand zwischen Nodes in Pixeln fuer Grid-Decimation (0.0 = deaktiviert).
    #[serde(default = "default_node_decimation_spacing_px")]
    pub node_decimation_spacing_px: f32,
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
            selection_style: SelectionStyle::default(),
            segment_stop_at_junction: default_segment_stop_at_junction(),
            segment_max_angle_deg: default_segment_max_angle_deg(),
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
            marker_outline_width: MARKER_OUTLINE_WIDTH,
            camera_zoom_min: CAMERA_ZOOM_MIN,
            camera_zoom_max: CAMERA_ZOOM_MAX,
            camera_zoom_step: CAMERA_ZOOM_STEP,
            camera_scroll_zoom_step: CAMERA_SCROLL_ZOOM_STEP,
            snap_scale_percent: SNAP_SCALE_PERCENT,
            hitbox_scale_percent: HITBOX_SCALE_PERCENT,
            mouse_wheel_distance_step_m: MOUSE_WHEEL_DISTANCE_STEP_M,
            value_adjust_input_mode: ValueAdjustInputMode::default(),
            reconnect_on_delete: true,
            split_connection_on_place: true,
            auto_create_segment: false,
            terrain_height_scale: TERRAIN_HEIGHT_SCALE,
            bg_opacity: 1.0,
            bg_opacity_at_min_zoom: 0.0,
            bg_fade_start_zoom: 3.5,
            copy_preview_opacity: default_copy_preview_opacity(),
            segment_lock_icon_size_px: default_segment_lock_icon_size_px(),
            overview_layers: OverviewLayerOptions::default(),
            zoom_compensation_max: DEFAULT_ZOOM_COMPENSATION_MAX,
            min_node_size_px: MIN_NODE_SIZE_PX,
            min_connection_width_px: MIN_CONNECTION_WIDTH_PX,
            min_arrow_size_px: MIN_ARROW_SIZE_PX,
            min_marker_size_px: MIN_MARKER_SIZE_PX,
            node_decimation_spacing_px: NODE_DECIMATION_SPACING_PX,
        }
    }
}

fn default_segment_stop_at_junction() -> bool {
    true
}

fn default_segment_max_angle_deg() -> f32 {
    15.0
}

fn default_copy_preview_opacity() -> f32 {
    0.5
}

fn default_segment_lock_icon_size_px() -> f32 {
    16.0
}

fn default_snap_scale_percent() -> f32 {
    SNAP_SCALE_PERCENT
}

fn default_hitbox_scale_percent() -> f32 {
    HITBOX_SCALE_PERCENT
}

fn default_mouse_wheel_distance_step_m() -> f32 {
    MOUSE_WHEEL_DISTANCE_STEP_M
}

fn default_marker_outline_width() -> f32 {
    MARKER_OUTLINE_WIDTH
}

fn default_camera_zoom_min() -> f32 {
    CAMERA_ZOOM_MIN
}

fn default_camera_zoom_max() -> f32 {
    CAMERA_ZOOM_MAX
}

fn default_bg_opacity() -> f32 {
    1.0
}

fn default_bg_opacity_at_min_zoom() -> f32 {
    0.0
}

fn default_bg_fade_start_zoom() -> f32 {
    3.5
}

fn default_zoom_compensation_max() -> f32 {
    DEFAULT_ZOOM_COMPENSATION_MAX
}

fn default_min_node_size_px() -> f32 {
    MIN_NODE_SIZE_PX
}

fn default_min_connection_width_px() -> f32 {
    MIN_CONNECTION_WIDTH_PX
}

fn default_min_arrow_size_px() -> f32 {
    MIN_ARROW_SIZE_PX
}

fn default_min_marker_size_px() -> f32 {
    MIN_MARKER_SIZE_PX
}

fn default_node_decimation_spacing_px() -> f32 {
    NODE_DECIMATION_SPACING_PX
}

impl EditorOptions {
    /// Laedt Optionen aus einer TOML-Datei. Bei Fehler: Standardwerte.
    pub fn load_from_file(path: &std::path::Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => match toml::from_str::<EditorOptions>(&content) {
                Ok(mut opts) => {
                    if opts.selection_size_factor > 0.0 && opts.selection_size_factor <= 5.0 {
                        opts.selection_size_factor *= 100.0;
                    }

                    if let Err(e) = opts.validate() {
                        log::warn!(
                            "Optionen-Validierung fehlgeschlagen, verwende Standardwerte: {}",
                            e
                        );
                        return Self::default();
                    }
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
        self.validate()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        log::info!("Optionen gespeichert nach: {}", path.display());
        Ok(())
    }

    /// Validiert EditorOptions auf Konsistenz.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.camera_zoom_min >= self.camera_zoom_max {
            return Err(anyhow::anyhow!(
                "camera_zoom_min ({}) muss < camera_zoom_max ({}) sein",
                self.camera_zoom_min,
                self.camera_zoom_max
            ));
        }

        if self.node_size_world <= 0.0 {
            return Err(anyhow::anyhow!("node_size_world muss > 0 sein"));
        }

        if self.hitbox_scale_percent < 25.0 || self.hitbox_scale_percent > 500.0 {
            return Err(anyhow::anyhow!(
                "hitbox_scale_percent ({}) muss zwischen 25 und 500 liegen",
                self.hitbox_scale_percent
            ));
        }

        if self.selection_size_factor < 100.0 || self.selection_size_factor > 200.0 {
            return Err(anyhow::anyhow!(
                "selection_size_factor ({}) muss zwischen 100 und 200 liegen",
                self.selection_size_factor
            ));
        }

        if self.snap_scale_percent < 25.0 || self.snap_scale_percent > 2000.0 {
            return Err(anyhow::anyhow!(
                "snap_scale_percent ({}) muss zwischen 25 und 2000 liegen",
                self.snap_scale_percent
            ));
        }

        if self.mouse_wheel_distance_step_m <= 0.0 || self.mouse_wheel_distance_step_m > 10.0 {
            return Err(anyhow::anyhow!(
                "mouse_wheel_distance_step_m ({}) muss > 0 und <= 10 sein",
                self.mouse_wheel_distance_step_m
            ));
        }

        if self.copy_preview_opacity < 0.0 || self.copy_preview_opacity > 1.0 {
            return Err(anyhow::anyhow!(
                "copy_preview_opacity ({}) muss zwischen 0 und 1 liegen",
                self.copy_preview_opacity
            ));
        }

        if self.segment_lock_icon_size_px <= 0.0 {
            return Err(anyhow::anyhow!(
                "segment_lock_icon_size_px ({}) muss > 0 sein",
                self.segment_lock_icon_size_px
            ));
        }

        // LOD-Mindestgroessen duerfen nicht negativ sein
        if self.min_node_size_px < 0.0 {
            return Err(anyhow::anyhow!("min_node_size_px darf nicht negativ sein"));
        }
        if self.min_connection_width_px < 0.0 {
            return Err(anyhow::anyhow!(
                "min_connection_width_px darf nicht negativ sein"
            ));
        }
        if self.min_arrow_size_px < 0.0 {
            return Err(anyhow::anyhow!("min_arrow_size_px darf nicht negativ sein"));
        }
        if self.min_marker_size_px < 0.0 {
            return Err(anyhow::anyhow!(
                "min_marker_size_px darf nicht negativ sein"
            ));
        }
        if self.node_decimation_spacing_px < 0.0 {
            return Err(anyhow::anyhow!(
                "node_decimation_spacing_px darf nicht negativ sein"
            ));
        }

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
    pub fn hitbox_radius(&self) -> f32 {
        self.node_size_world * self.hitbox_scale_percent / 100.0
    }

    /// Berechnet den Snap-Radius in Welteinheiten.
    pub fn snap_radius(&self) -> f32 {
        self.node_size_world * self.snap_scale_percent / 100.0
    }

    /// Berechnet den Selektions-Multiplikator aus `selection_size_factor` in Prozent.
    pub fn selection_size_multiplier(&self) -> f32 {
        self.selection_size_factor / 100.0
    }

    /// Berechnet den Zoom-Kompensationsfaktor fuer eine gegebene Zoom-Stufe.
    ///
    /// Formel: `(1/zoom)^0.5`, geclampt auf `[1.0, zoom_compensation_max]`.
    /// Bei `zoom >= 1.0` ist der Faktor `1.0` (keine zusaetzliche Vergroesserung).
    /// Bei `zoom_compensation_max <= 1.0` ist die Kompensation deaktiviert.
    pub fn zoom_compensation(&self, zoom: f32) -> f32 {
        if self.zoom_compensation_max <= 1.0 {
            return 1.0;
        }
        (1.0 / zoom.max(f32::EPSILON))
            .powf(0.5)
            .clamp(1.0, self.zoom_compensation_max)
    }

    /// Berechnet die Grid-Zellgroesse fuer die Node-Decimation in Welteinheiten.
    ///
    /// Gibt `0.0` zurueck, wenn die Decimation deaktiviert ist (`node_decimation_spacing_px == 0`).
    /// Die Zellgroesse skaliert automatisch mit dem Zoom-Level.
    pub fn decimation_cell_size(&self, world_per_pixel: f32) -> f32 {
        if self.node_decimation_spacing_px <= 0.0 {
            return 0.0;
        }
        self.node_decimation_spacing_px * world_per_pixel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Prüft, dass eine alte TOML-Datei ohne neue Felder korrekt mit Defaults geladen wird.
    #[test]
    fn test_deserialize_missing_new_fields_uses_defaults() {
        // Minimale gueltige TOML ohne auto_create_segment und marker_outline_width
        let toml_str = r#"
node_size_world = 1.0
node_color_default = [0.2, 0.6, 1.0, 1.0]
node_color_subprio = [0.5, 0.5, 0.5, 1.0]
node_color_selected = [1.0, 0.0, 0.0, 1.0]
node_color_warning = [1.0, 1.0, 0.0, 1.0]
selection_size_factor = 140.0
connection_thickness_world = 0.3
connection_thickness_subprio_world = 0.15
arrow_length_world = 1.5
arrow_width_world = 0.8
connection_color_regular = [0.2, 0.6, 1.0, 1.0]
connection_color_dual = [0.0, 1.0, 0.5, 1.0]
connection_color_reverse = [1.0, 0.3, 0.3, 1.0]
marker_size_world = 3.0
marker_color = [1.0, 0.0, 0.0, 1.0]
marker_outline_color = [0.0, 0.0, 0.0, 1.0]
camera_zoom_step = 1.15
camera_scroll_zoom_step = 1.05
terrain_height_scale = 1.0
"#;
        let opts: EditorOptions =
            toml::from_str(toml_str).expect("Deserialisierung fehlgeschlagen");
        assert_eq!(
            opts.auto_create_segment, false,
            "auto_create_segment muss default false sein"
        );
        assert!(
            (opts.marker_outline_width - MARKER_OUTLINE_WIDTH).abs() < f32::EPSILON,
            "marker_outline_width muss default {} sein, ist {}",
            MARKER_OUTLINE_WIDTH,
            opts.marker_outline_width
        );
    }

    /// Prüft, dass Roundtrip serialize → deserialize die neuen Felder erhält.
    #[test]
    fn test_toml_roundtrip_new_fields() {
        let mut opts = EditorOptions::default();
        opts.auto_create_segment = false;
        opts.marker_outline_width = 0.15;

        let toml_str = toml::to_string_pretty(&opts).expect("Serialisierung fehlgeschlagen");
        let loaded: EditorOptions =
            toml::from_str(&toml_str).expect("Deserialisierung fehlgeschlagen");

        assert_eq!(
            loaded.auto_create_segment, false,
            "auto_create_segment muss false bleiben"
        );
        assert!(
            (loaded.marker_outline_width - 0.15).abs() < f32::EPSILON,
            "marker_outline_width muss 0.15 bleiben"
        );
    }
}
