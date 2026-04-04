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
    /// Monotone Revision fuer Bildinhalt/Existenz des Background-Assets.
    pub background_asset_revision: u64,
    /// Monotone Revision fuer Platzierung/Skalierung des Background-Assets.
    pub background_transform_revision: u64,
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
            background_asset_revision: 0,
            background_transform_revision: 0,
        }
    }

    /// Markiert, dass sich Bildinhalt oder Existenz des Background-Assets geaendert haben.
    pub fn mark_background_asset_changed(&mut self) {
        self.background_asset_revision = self.background_asset_revision.saturating_add(1);
        self.mark_background_transform_changed();
    }

    /// Markiert, dass sich Bounds oder Skalierung des Background-Assets geaendert haben.
    pub fn mark_background_transform_changed(&mut self) {
        self.background_transform_revision = self.background_transform_revision.saturating_add(1);
    }
}
