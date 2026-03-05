//! Use-Cases der Application-Layer-Orchestrierung.

/// Automatische Erkennung von Heightmap, Overview-Bild und Map-Mod-ZIP nach Laden.
pub mod auto_detect;
/// Use-Cases fuer Background-Map (Laden, Sichtbarkeit, Skalierung, ZIP).
pub mod background_map;
/// Use-Case-Funktionen fuer Kamera-Steuerung (Pan, Zoom, Zentrieren).
pub mod camera;
/// Use-Case-Funktionen fuer Node/Connection-Editing (Add, Delete, Connect, …).
pub mod editing;
/// Use-Case-Funktionen fuer Dateisystem-Operationen (Laden, Speichern, Dedup).
pub mod file_io;
/// Use-Cases fuer Heightmap-Verwaltung (Setzen, Dialog, Warnung).
pub mod heightmap;
/// Use-Case-Funktionen fuer Node-Selektion (Pick, Rect, Lasso, Move).
pub mod selection;
/// Use-Cases fuer Viewport-Groesse und Render-Qualitaet.
pub mod viewport;
