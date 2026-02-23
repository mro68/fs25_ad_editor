//! Core-Domänentypen: Nodes, Connections, RoadMap, Kamera, Spatial-Index.

pub mod background_map;
pub mod camera;
pub mod connection;
pub mod heightmap;
pub mod map_marker;
pub mod meta;
/// Core-Datenmodelle für AutoDrive-Konfigurationen
///
/// Dieses Modul definiert die Haupt-Datenstrukturen:
/// - RoadMap: Container für alle Nodes und Connections
/// - MapNode: Einzelner Wegpunkt mit Position und Eigenschaften
/// - Connection: Verbindung zwischen zwei Nodes
pub mod node;
pub mod road_map;
pub mod spatial;

pub use background_map::BackgroundMap;
pub use background_map::{list_images_in_zip, load_from_zip};
pub use camera::Camera2D;
pub use connection::{Connection, ConnectionDirection, ConnectionPriority};
pub use heightmap::{Heightmap, WorldBounds};
pub use map_marker::MapMarker;
pub use meta::AutoDriveMeta;
pub use node::{MapNode, NodeFlag};
pub use road_map::{ConnectedNeighbor, DeduplicationResult, RoadMap};
pub use spatial::{SpatialIndex, SpatialMatch};
