use crate::core::{BackgroundMap, Camera2D};
use crate::shared::RenderQuality;
use std::sync::Arc;

/// View-bezogener Anwendungszustand
#[derive(Default)]
pub struct ViewState {
    /// 2D-Kamera fuer die Ansicht
    pub camera: Camera2D,
    /// Aktuelle Viewport-Groesse in Pixel
    pub viewport_size: [f32; 2],
    /// Qualitaetsstufe fuer Kantenglaettung
    pub render_quality: RenderQuality,
    /// Background-Map (optional)
    pub background_map: Option<Arc<BackgroundMap>>,
    /// Background-Sichtbarkeit
    pub background_visible: bool,
    /// Skalierungsfaktor fuer Background-Map-Ausdehnung (1.0 = Original)
    pub background_scale: f32,
    /// Signalisiert, dass die Background-Map neu in den GPU-Renderer hochgeladen werden muss
    pub background_dirty: bool,
    /// Ob das Straßenoverlay auf dem Hintergrundbild angezeigt wird.
    ///
    /// Nur wirksam wenn `AppState.road_mask` gesetzt ist.
    pub show_road_overlay: bool,
}

impl ViewState {
    /// Erstellt den Standard-View-Zustand.
    pub fn new() -> Self {
        Self {
            camera: Camera2D::new(),
            viewport_size: [0.0, 0.0],
            render_quality: RenderQuality::High,
            background_map: None,
            background_visible: true,
            background_scale: 1.0,
            background_dirty: false,
            show_road_overlay: false,
        }
    }
}
