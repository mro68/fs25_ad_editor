//! Render-Szene als expliziter Übergabevertrag zwischen App und Renderer.
//!
//! Lebt im shared-Modul, da `app` sie baut und `render` sie konsumiert.

use super::options::EditorOptions;
use super::RenderQuality;
use crate::core::{BackgroundMap, Camera2D, RoadMap};
use std::collections::HashSet;
use std::sync::Arc;

/// Read-only Daten für einen Render-Frame.
#[derive(Clone)]
pub struct RenderScene {
    /// Die aktuelle RoadMap (Nodes + Connections)
    pub road_map: Option<Arc<RoadMap>>,
    /// Kamera-Zustand für diesen Frame
    pub camera: Camera2D,
    /// Viewport-Größe in Pixeln [Breite, Höhe]
    pub viewport_size: [f32; 2],
    /// Render-Qualitätsstufe (Anti-Aliasing)
    pub render_quality: RenderQuality,
    /// IDs der aktuell selektierten Nodes (Arc für O(1)-Clone pro Frame)
    pub selected_node_ids: Arc<HashSet<u64>>,
    /// Node-ID des Connect-Tool-Source (für spezielle Hervorhebung)
    pub connect_source_node: Option<u64>,
    /// Background-Map (optional)
    pub background_map: Option<Arc<BackgroundMap>>,
    /// Background-Opacity (0.0 = transparent, 1.0 = opak)
    pub background_opacity: f32,
    /// Background-Sichtbarkeit
    pub background_visible: bool,
    /// Laufzeit-Optionen für Farben, Größen, Breiten
    pub options: EditorOptions,
    /// Node-IDs, die im aktuellen Frame ausgeblendet werden sollen (z.B. Distanzen-Vorschau)
    pub hidden_node_ids: Arc<HashSet<u64>>,
}

impl RenderScene {
    /// Gibt zurück, ob eine Karte für Rendering vorhanden ist.
    pub fn has_map(&self) -> bool {
        self.road_map.is_some()
    }
}
