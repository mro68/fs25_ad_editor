//! Render-Szene als expliziter Uebergabevertrag zwischen App und Renderer.
//!
//! Lebt im shared-Modul, da `app` sie baut und `render` sie konsumiert.

use super::options::{EditorOptions, CAMERA_BASE_WORLD_EXTENT};
use super::RenderQuality;
use glam::{Mat3, Vec2};
use indexmap::IndexSet;
use kiddo::{ImmutableKdTree, SquaredEuclidean};
use std::collections::HashMap;
use std::sync::Arc;

/// Render-seitige Klassifikation eines Nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RenderNodeKind {
    /// Standard-Node ohne besondere Warn- oder Subprio-Faerbung.
    Regular,
    /// Subpriorisierter Node.
    SubPrio,
    /// Warn-Node.
    Warning,
}

/// Render-seitige Node-Daten ohne Domain-Abhaengigkeit.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RenderNode {
    /// Stabile Node-ID fuer Auswahl- und Sichtbarkeitsmengen.
    pub id: u64,
    /// Weltposition des Nodes.
    pub position: Vec2,
    /// Rendering-Klassifikation fuer Farben.
    pub kind: RenderNodeKind,
    /// Nodes, die auch bei Decimation sichtbar bleiben muessen.
    pub preserve_when_decimating: bool,
}

/// Render-seitige Richtungsklassifikation einer Verbindung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RenderConnectionDirection {
    /// Pfeil in Start-zu-Ende-Richtung.
    Regular,
    /// Bidirektionale Verbindung ohne Pfeil.
    Dual,
    /// Pfeil entgegengesetzt zur Start-zu-Ende-Geometrie.
    Reverse,
}

/// Render-seitige Prioritaetsklassifikation einer Verbindung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RenderConnectionPriority {
    /// Normale Verbindung.
    Regular,
    /// Subpriorisierte Verbindung.
    SubPriority,
}

/// Render-seitige Verbindung mit bereits aufgeloester Geometrie.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RenderConnection {
    /// Start-Node-ID fuer Hidden-Filtering.
    pub start_id: u64,
    /// End-Node-ID fuer Hidden-Filtering.
    pub end_id: u64,
    /// Startposition der Verbindung.
    pub start_pos: Vec2,
    /// Endposition der Verbindung.
    pub end_pos: Vec2,
    /// Render-seitige Richtungsklassifikation.
    pub direction: RenderConnectionDirection,
    /// Render-seitige Prioritaetsklassifikation.
    pub priority: RenderConnectionPriority,
}

/// Render-seitige Marker-Daten mit bereits aufgeloester Position.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RenderMarker {
    /// Weltposition des Markers.
    pub position: Vec2,
}

#[derive(Debug, Clone)]
struct RenderSpatialIndex {
    tree: ImmutableKdTree<f64, 2>,
    node_ids: Vec<u64>,
    positions: HashMap<u64, Vec2>,
}

impl RenderSpatialIndex {
    fn empty() -> Self {
        Self {
            tree: ImmutableKdTree::new_from_slice(&[]),
            node_ids: Vec::new(),
            positions: HashMap::new(),
        }
    }

    fn from_nodes(nodes: &HashMap<u64, RenderNode>) -> Self {
        if nodes.is_empty() {
            return Self::empty();
        }

        let mut node_ids: Vec<u64> = nodes.keys().copied().collect();
        node_ids.sort_unstable();

        let entries: Vec<[f64; 2]> = node_ids
            .iter()
            .filter_map(|id| {
                nodes
                    .get(id)
                    .map(|node| [node.position.x as f64, node.position.y as f64])
            })
            .collect();

        let tree: ImmutableKdTree<f64, 2> = entries.as_slice().into();
        let positions = nodes
            .iter()
            .map(|(id, node)| (*id, node.position))
            .collect();

        Self {
            tree,
            node_ids,
            positions,
        }
    }

    fn within_rect_into(&self, min: Vec2, max: Vec2, out: &mut Vec<u64>) {
        out.clear();
        if self.node_ids.is_empty() {
            return;
        }

        let center_x = (min.x + max.x) as f64 * 0.5;
        let center_y = (min.y + max.y) as f64 * 0.5;
        let half_w = (max.x - min.x) as f64 * 0.5;
        let half_h = (max.y - min.y) as f64 * 0.5;
        let radius_sq = half_w * half_w + half_h * half_h;

        for entry in self
            .tree
            .within::<SquaredEuclidean>(&[center_x, center_y], radius_sq)
        {
            if let Some(&node_id) = self.node_ids.get(entry.item as usize) {
                if let Some(pos) = self.positions.get(&node_id) {
                    if pos.x >= min.x && pos.x <= max.x && pos.y >= min.y && pos.y <= max.y {
                        out.push(node_id);
                    }
                }
            }
        }
    }
}

/// Render-spezifischer Snapshot einer Karte.
#[derive(Debug)]
pub(crate) struct RenderMap {
    nodes: HashMap<u64, RenderNode>,
    connections: Vec<RenderConnection>,
    markers: Vec<RenderMarker>,
    spatial_index: RenderSpatialIndex,
}

impl RenderMap {
    pub(crate) fn new(
        nodes: HashMap<u64, RenderNode>,
        connections: Vec<RenderConnection>,
        markers: Vec<RenderMarker>,
    ) -> Self {
        let spatial_index = RenderSpatialIndex::from_nodes(&nodes);
        Self {
            nodes,
            connections,
            markers,
            spatial_index,
        }
    }

    pub(crate) fn node(&self, node_id: &u64) -> Option<&RenderNode> {
        self.nodes.get(node_id)
    }

    pub(crate) fn nodes_within_rect_into(&self, min: Vec2, max: Vec2, out: &mut Vec<u64>) {
        self.spatial_index.within_rect_into(min, max, out);
    }

    pub(crate) fn connections(&self) -> &[RenderConnection] {
        &self.connections
    }

    pub(crate) fn markers(&self) -> &[RenderMarker] {
        &self.markers
    }

    pub(crate) fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn marker_count(&self) -> usize {
        self.markers.len()
    }
}

/// Render-seitige Kameradaten ohne Core-Abhaengigkeit.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RenderCamera {
    /// Kameraposition in Weltkoordinaten.
    pub position: Vec2,
    /// Zoom-Faktor des Frames.
    pub zoom: f32,
}

impl RenderCamera {
    pub(crate) fn new(position: Vec2, zoom: f32) -> Self {
        Self { position, zoom }
    }

    pub(crate) fn view_matrix(&self) -> Mat3 {
        Mat3::from_translation(-self.position)
    }

    pub(crate) fn world_per_pixel(&self, viewport_height: f32) -> f32 {
        2.0 * CAMERA_BASE_WORLD_EXTENT / (self.zoom * viewport_height.max(1.0))
    }
}

#[derive(Clone)]
pub(crate) struct RenderSceneFrameData {
    pub camera: RenderCamera,
    pub viewport_size: [f32; 2],
    pub render_quality: RenderQuality,
    pub selected_node_ids: Arc<IndexSet<u64>>,
    pub has_background: bool,
    pub background_visible: bool,
    pub options: Arc<EditorOptions>,
    pub hidden_node_ids: Arc<IndexSet<u64>>,
    pub dimmed_node_ids: Arc<IndexSet<u64>>,
}

/// Read-only Daten fuer einen Render-Frame.
#[derive(Clone)]
pub struct RenderScene {
    map: Option<Arc<RenderMap>>,
    camera: RenderCamera,
    viewport_size: [f32; 2],
    render_quality: RenderQuality,
    selected_node_ids: Arc<IndexSet<u64>>,
    has_background: bool,
    background_visible: bool,
    options: Arc<EditorOptions>,
    hidden_node_ids: Arc<IndexSet<u64>>,
    dimmed_node_ids: Arc<IndexSet<u64>>,
}

impl RenderScene {
    pub(crate) fn new(map: Option<Arc<RenderMap>>, frame: RenderSceneFrameData) -> Self {
        Self {
            map,
            camera: frame.camera,
            viewport_size: frame.viewport_size,
            render_quality: frame.render_quality,
            selected_node_ids: frame.selected_node_ids,
            has_background: frame.has_background,
            background_visible: frame.background_visible,
            options: frame.options,
            hidden_node_ids: frame.hidden_node_ids,
            dimmed_node_ids: frame.dimmed_node_ids,
        }
    }

    /// Gibt zurueck, ob eine Karte fuer Rendering vorhanden ist.
    pub fn has_map(&self) -> bool {
        self.map.is_some()
    }

    /// Gibt zurueck, ob ein Hintergrundbild fuer den Frame vorhanden ist.
    pub fn has_background(&self) -> bool {
        self.has_background
    }

    pub(crate) fn map(&self) -> Option<&RenderMap> {
        self.map.as_deref()
    }

    pub(crate) fn camera(&self) -> &RenderCamera {
        &self.camera
    }

    pub(crate) fn viewport_size(&self) -> [f32; 2] {
        self.viewport_size
    }

    pub(crate) fn render_quality(&self) -> RenderQuality {
        self.render_quality
    }

    pub(crate) fn selected_node_ids(&self) -> &IndexSet<u64> {
        self.selected_node_ids.as_ref()
    }

    pub(crate) fn background_visible(&self) -> bool {
        self.background_visible
    }

    pub(crate) fn options(&self) -> &EditorOptions {
        self.options.as_ref()
    }

    pub(crate) fn hidden_node_ids(&self) -> &IndexSet<u64> {
        self.hidden_node_ids.as_ref()
    }

    pub(crate) fn dimmed_node_ids(&self) -> &IndexSet<u64> {
        self.dimmed_node_ids.as_ref()
    }
}
