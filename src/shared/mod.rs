//! Geteilte Typen fuer layer-uebergreifende Vertraege.
//!
//! Enthaelt Typen, die zwischen `app` und `render` geteilt werden,
//! um direkte Abhaengigkeiten zu vermeiden.

/// Gemeinsame Geometrie-Hilfsfunktionen (Winkelberechnung, etc.).
pub mod geometry;
/// Mehrsprachigkeits-System (Language-Enum, I18nKey, t()-Funktion).
pub mod i18n;
/// Konfigurationsoptionen (EditorOptions, RenderQuality, Farben, Kamera-Parameter).
pub mod options;
mod render_quality;
mod render_scene;
/// Spline-Geometrie-Hilfsfunktionen (Catmull-Rom, Arc-Length-Resampling).
pub mod spline_geometry;

pub use geometry::angle_deviation;
pub use i18n::{t, I18nKey, Language};
pub use options::EditorOptions;
pub use options::OverviewLayerOptions;
pub use options::SelectionStyle;
pub use options::ValueAdjustInputMode;
pub use options::{SNAP_SCALE_PERCENT, TERRAIN_HEIGHT_SCALE};
pub use render_quality::RenderQuality;
pub use render_scene::RenderScene;
