//! Core-Domaenentypen: Nodes, Connections, RoadMap, Kamera, Spatial-Index.

/// Hintergrundkarten-Loader fuer Map-Rendering (PNG, JPG, DDS, ZIP).
pub mod background_map;
/// 2D-Kamera mit Pan und Zoom fuer den Viewport.
pub mod camera;
/// Centerline-Berechnung fuer Feldkorridore (polygon-, segment- und rasterbasiert).
pub mod centerline;
/// Verbindungen zwischen Wegpunkten (Richtung, Prioritaet, Geometrie).
pub mod connection;
/// Feldgrenz-Polygone in Weltkoordinaten (aus GRLE-Farmland-Daten).
pub mod farmland;
/// Heightmap-Loader und Y-Koordinaten-Sampling.
pub mod heightmap;
/// Benannte Wegpunkt-Marker aus der AutoDrive-Konfiguration.
pub mod map_marker;
/// Nicht-renderrelevante Metadaten aus der AutoDrive-XML-Konfiguration.
pub mod meta;
/// Wegpunkt-Typen und Flags fuer das AutoDrive-Netzwerk.
pub mod node;
/// Zentrales Straßennetz-Datenmodell mit Nodes, Connections und Spatial-Index.
pub mod road_map;
/// Spatial-Index (KD-Tree) fuer schnelle Node-Abfragen.
pub mod spatial;
/// Zhang-Suen-Thinning: Skelettierung von Binaermasken.
pub mod thinning;

pub use background_map::BackgroundMap;
pub use background_map::{list_images_in_zip, load_from_zip, ZipImageEntry};
pub use camera::Camera2D;
pub use centerline::{
    compute_polygon_centerline, compute_segment_centerline, compute_voronoi_bfs,
    extract_boundary_centerline, extract_corridor_centerline, VoronoiGrid,
};
pub use connection::{Connection, ConnectionDirection, ConnectionPriority};
pub use farmland::{
    find_polygon_at, offset_polygon, point_in_polygon, simplify_polygon, simplify_polyline,
    FarmlandGrid, FieldPolygon,
};
pub use heightmap::{Heightmap, WorldBounds};
pub use map_marker::MapMarker;
pub use meta::AutoDriveMeta;
pub use node::{MapNode, NodeFlag};
pub use road_map::{BoundaryNode, ConnectedNeighbor, DeduplicationResult, RoadMap};
pub use spatial::{SpatialIndex, SpatialMatch};
pub use thinning::zhang_suen_thinning;
