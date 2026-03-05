//! Feature-Handler fuer AppCommand-Verarbeitung.
//!
//! Jeder Handler gruppiert die Command-Ausfuehrung eines Feature-Bereichs.
//! Der Controller dispatcht an die passende Handler-Funktion.

/// Handler fuer Dialog-State und Anwendungssteuerung (Exit, Optionen, Marker).
pub mod dialog;
/// Handler fuer Node/Connection-Editing, Marker und Editor-Werkzeug.
pub mod editing;
/// Handler fuer Datei-Operationen (Oeffnen, Speichern, Heightmap, Dedup).
pub mod file_io;
/// Zentrale Helfer fuer Undo/Selection-Snapshots in den Handlern.
pub mod helpers;
/// Handler fuer Undo/Redo-Operationen.
pub mod history;
/// Handler fuer Route-Tool-Operationen (Linie, Parkplatz, Kurve, …).
pub mod route_tool;
/// Handler fuer Selektions-Operationen (Pick, Rect, Lasso, Move).
pub mod selection;
/// Handler fuer Kamera, Viewport und Background-Map.
pub mod view;
