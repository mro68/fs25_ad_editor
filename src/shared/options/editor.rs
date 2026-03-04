//! Laufzeitoptionen (`EditorOptions`) inkl. Persistenz und Validierung.

use super::camera::{CAMERA_SCROLL_ZOOM_STEP, CAMERA_ZOOM_MAX, CAMERA_ZOOM_MIN, CAMERA_ZOOM_STEP};
use super::render::{
    OverviewLayerOptions, SelectionStyle, ARROW_LENGTH_WORLD, ARROW_WIDTH_WORLD,
    CONNECTION_COLOR_DUAL, CONNECTION_COLOR_REGULAR, CONNECTION_COLOR_REVERSE,
    CONNECTION_THICKNESS_SUBPRIO_WORLD, CONNECTION_THICKNESS_WORLD, MARKER_COLOR,
    MARKER_OUTLINE_COLOR, MARKER_SIZE_WORLD, NODE_COLOR_DEFAULT, NODE_COLOR_SELECTED,
    NODE_COLOR_SUBPRIO, NODE_COLOR_WARNING, NODE_SIZE_WORLD, SELECTION_SIZE_FACTOR,
    TERRAIN_HEIGHT_SCALE,
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

    // Uebersichtskarte
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
            selection_style: SelectionStyle::default(),
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
            snap_scale_percent: SNAP_SCALE_PERCENT,
            hitbox_scale_percent: HITBOX_SCALE_PERCENT,
            mouse_wheel_distance_step_m: MOUSE_WHEEL_DISTANCE_STEP_M,
            value_adjust_input_mode: ValueAdjustInputMode::default(),
            reconnect_on_delete: true,
            split_connection_on_place: true,
            terrain_height_scale: TERRAIN_HEIGHT_SCALE,
            bg_opacity: 1.0,
            bg_opacity_at_min_zoom: 0.0,
            bg_fade_start_zoom: 3.5,
            overview_layers: OverviewLayerOptions::default(),
        }
    }
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
}
