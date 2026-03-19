//! Kamera-bezogene Standardwerte fuer den Editor.

/// Sichtbare Welt-Halbbreite bei Zoom 1.0 (Einheiten = AutoDrive-Meter).
pub const CAMERA_BASE_WORLD_EXTENT: f32 = 2048.0;
/// Minimaler Zoom-Faktor.
pub const CAMERA_ZOOM_MIN: f32 = 0.75;
/// Maximaler Zoom-Faktor.
pub const CAMERA_ZOOM_MAX: f32 = 200.0;
/// Zoom-Schritt bei stufenweisem Zoom (Menue-Buttons / Shortcuts).
pub const CAMERA_ZOOM_STEP: f32 = 1.1;
/// Zoom-Schritt bei Mausrad-Scroll.
pub const CAMERA_SCROLL_ZOOM_STEP: f32 = 1.045;
