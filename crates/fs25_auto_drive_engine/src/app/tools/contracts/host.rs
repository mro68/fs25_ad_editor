//! Host-Synchronisation fuer Route-Tools.

use std::sync::Arc;

use image::DynamicImage;

use crate::core::{ConnectionDirection, ConnectionPriority, FarmlandGrid, FieldPolygon};

/// Gebuendelter Host-Kontext fuer aktive Route-Tools.
#[derive(Clone)]
pub struct ToolHostContext {
    /// Editor-Standardrichtung fuer neue Verbindungen.
    pub direction: ConnectionDirection,
    /// Editor-Standardprioritaet fuer neue Verbindungen.
    pub priority: ConnectionPriority,
    /// Aktueller Snap-Radius in Weltkoordinaten.
    pub snap_radius: f32,
    /// Optional geladene Farmland-Polygone.
    pub farmland_data: Option<Arc<Vec<FieldPolygon>>>,
    /// Optional geladenes Farmland-Raster.
    pub farmland_grid: Option<Arc<FarmlandGrid>>,
    /// Optional geladenes Hintergrundbild.
    pub background_image: Option<Arc<DynamicImage>>,
}

/// Synchronisiert Editor-Defaults und externe Assets in ein Tool.
pub trait RouteToolHostSync {
    /// Uebernimmt den aktuellen Host-Kontext.
    fn sync_host(&mut self, context: &ToolHostContext);
}
