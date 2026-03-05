//! UI-Komponenten: Menue, Toolbar, Properties, Input-Handling, Dialoge.

/// Kontext-Menue fuer Rechtsklick-Aktionen im Viewport.
pub mod context_menu;
/// Panel fuer Default-Werte neuer Verbindungen (Richtung, Prioritaet).
pub mod defaults_panel;
/// Alle Dialoge (Datei-IO, Dedup, Marker, Heightmap, Uebersichtskarte).
pub mod dialogs;
mod drag;
/// Editor-Panel fuer die Bearbeitung selektierter Knoten und Verbindungen.
pub mod edit_panel;
/// Viewport-Input-Verarbeitung (Drag, Scroll, Mausklick, Selektion).
pub mod input;
mod keyboard;
/// Menue-Leiste mit Datei-, Bearbeitungs- und Ansicht-Aktionen.
pub mod menu;
/// Optionen-Dialog fuer Editor-Einstellungen.
pub mod options_dialog;
/// Properties-Panel fuer selektierte Nodes und Verbindungen.
pub mod properties;
/// Segment-Overlay: Rahmen und Lock-Icons fuer registrierte Segmente.
pub mod segment_overlay;
/// Statusleiste mit Anzeige des aktuellen Editor-Zustands.
pub mod status;
/// Live-Vorschau aktiver Werkzeuge im Viewport (Overlay-Rendering).
pub mod tool_preview;
/// Werkzeug-Toolbar fuer Werkzeugauswahl und -konfiguration.
pub mod toolbar;

pub use defaults_panel::render_route_defaults_panel;
pub use dialogs::{
    handle_file_dialogs, show_dedup_dialog, show_heightmap_warning, show_marker_dialog,
    show_overview_options_dialog, show_post_load_dialog, show_save_overview_dialog,
    show_zip_browser,
};
pub use edit_panel::render_edit_panel;
pub use input::InputState;
pub use menu::render_menu;
pub use options_dialog::show_options_dialog;
pub use properties::render_properties_panel;
pub use segment_overlay::{render_segment_overlays, SegmentOverlayEvent};
pub use status::render_status_bar;
pub use tool_preview::{
    paint_clipboard_preview, paint_preview, paint_preview_polyline, render_tool_preview,
};
pub use toolbar::render_toolbar;
