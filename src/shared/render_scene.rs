//! Render-Szene als expliziter Uebergabevertrag zwischen App und Renderer.
//!
//! Lebt im shared-Modul, da `app` sie baut und `render` sie konsumiert.
//!
//! Core-Typen werden ueber den Crate-Root eingebunden (nicht ueber direkte Submodul-Pfade),
//! da `shared` gemaess Architektur-Konvention keine direkten Submodul-Imports enthalten darf.

use super::options::EditorOptions;
use super::RenderQuality;
use crate::{BackgroundMap, Camera2D, RoadMap};
use indexmap::IndexSet;
use std::sync::Arc;

/// Read-only Daten fuer einen Render-Frame.
#[derive(Clone)]
pub struct RenderScene {
    /// Die aktuelle RoadMap (Nodes + Connections)
    pub road_map: Option<Arc<RoadMap>>,
    /// Kamera-Zustand fuer diesen Frame
    pub camera: Camera2D,
    /// Viewport-Groesse in Pixeln [Breite, Hoehe]
    pub viewport_size: [f32; 2],
    /// Render-Qualitaetsstufe (Anti-Aliasing)
    pub render_quality: RenderQuality,
    /// IDs der aktuell selektierten Nodes in Klick-Reihenfolge (Arc fuer O(1)-Clone pro Frame)
    pub selected_node_ids: Arc<IndexSet<u64>>,
    /// Node-ID des Connect-Tool-Source (fuer spezielle Hervorhebung)
    pub connect_source_node: Option<u64>,
    /// Background-Map (optional)
    pub background_map: Option<Arc<BackgroundMap>>,
    /// Background-Sichtbarkeit
    pub background_visible: bool,
    /// Laufzeit-Optionen fuer Farben, Groessen, Breiten (Arc fuer O(1)-Clone pro Frame)
    pub options: Arc<EditorOptions>,
    /// Node-IDs, die im aktuellen Frame ausgeblendet werden sollen (z.B. Distanzen-Vorschau)
    pub hidden_node_ids: Arc<IndexSet<u64>>,
}

impl RenderScene {
    /// Gibt zurueck, ob eine Karte fuer Rendering vorhanden ist.
    pub fn has_map(&self) -> bool {
        self.road_map.is_some()
    }
}
