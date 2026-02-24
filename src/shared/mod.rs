//! Geteilte Typen für layer-übergreifende Verträge.
//!
//! Enthält Typen, die zwischen `app` und `render` geteilt werden,
//! um direkte Abhängigkeiten zu vermeiden.
pub mod options;
mod render_quality;
mod render_scene;
pub mod spline_geometry;

pub use options::EditorOptions;
pub use options::OverviewLayerOptions;
pub use options::{SNAP_RADIUS, TERRAIN_HEIGHT_SCALE};
pub use render_quality::RenderQuality;
pub use render_scene::RenderScene;
