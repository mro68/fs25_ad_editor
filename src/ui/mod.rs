//! UI-Komponenten: Menue, Properties, Input-Handling, Dialoge.

/// Command Palette Overlay mit Suchfeld und Schnellaktionen.
pub mod command_palette;
/// Gemeinsame UI-Hilfsfunktionen (Wheel-Step, etc.).
pub mod common;
/// Kontext-Menue fuer Rechtsklick-Aktionen im Viewport.
pub mod context_menu;
/// Panel fuer Default-Werte neuer Verbindungen (Richtung, Prioritaet).
pub mod defaults_panel;
/// Alle Dialoge (Datei-IO, Dedup, Marker, Heightmap, Uebersichtskarte).
pub mod dialogs;
mod drag;
/// Editor-Panel fuer die Bearbeitung selektierter Knoten und Verbindungen.
pub mod edit_panel;
/// Schwebendes Kontextmenue fuer Werkzeuggruppen an der Mausposition.
pub mod floating_menu;
/// Gruppen-Boundary-Overlay: Ein-/Ausfahrt-Icons fuer Boundary-Nodes einer Gruppe.
pub mod group_boundary_overlay;
/// Segment-Overlay: Rahmen und Lock-Icons fuer registrierte Segmente.
pub mod group_overlay;
/// Gemeinsame Icon-Helfer fuer Tool-Buttons.
pub mod icons;
/// Viewport-Input-Verarbeitung (Drag, Scroll, Mausklick, Selektion).
pub mod input;
mod keyboard;
/// Wiederverwendbares Long-Press-Dropdown fuer Icon-Gruppen.
pub mod long_press;
/// Rechte Sidebar fuer Map-Marker (Kamera-Zentrierung bei Klick).
pub mod marker_panel;
/// Menue-Leiste mit Datei-, Bearbeitungs- und Ansicht-Aktionen.
pub mod menu;
/// Optionen-Dialog fuer Editor-Einstellungen.
pub mod options_dialog;
/// Properties-Panel fuer selektierte Nodes und Verbindungen.
pub mod properties;
/// Statusleiste mit Anzeige des aktuellen Editor-Zustands.
pub mod status;
/// Live-Vorschau aktiver Werkzeuge im Viewport (Overlay-Rendering).
pub mod tool_preview;
pub use defaults_panel::render_route_defaults_panel;
pub use dialogs::{
    handle_file_dialogs, show_confirm_dissolve_dialog, show_dedup_dialog,
    show_group_settings_popup, show_heightmap_warning, show_marker_dialog,
    show_overview_options_dialog, show_post_load_dialog, show_save_overview_dialog,
    show_trace_all_fields_dialog, show_zip_browser,
};
pub use edit_panel::render_edit_panel;
pub use floating_menu::render_floating_menu;
pub use group_boundary_overlay::{render_group_boundary_overlays, GroupBoundaryIcons};
pub use group_overlay::{render_group_overlays, GroupOverlayEvent};
pub use input::InputState;
pub use marker_panel::render_marker_content;
pub use menu::render_menu;
pub use options_dialog::show_options_dialog;
pub use properties::render_properties_content;
pub use status::render_status_bar;
pub use tool_preview::{
    paint_clipboard_preview, paint_preview, paint_preview_polyline, render_tool_preview,
};
