//! UI-Komponenten: Menü, Toolbar, Properties, Input-Handling, Dialoge.

mod context_menu;
pub mod dialogs;
mod drag;
pub mod input;
mod keyboard;
/// UI-Layer mit egui
///
/// Dieses Modul implementiert alle UI-Komponenten (Menüs, Panels, Dialogs).
/// Modulare Aufteilung: Keyboard-Shortcuts, Drag-Logik und Kontextmenüs
/// sind in eigene Dateien extrahiert.
pub mod menu;
pub mod options_dialog;
pub mod properties;
pub mod status;
pub mod toolbar;

pub use dialogs::{handle_file_dialogs, show_dedup_dialog, show_heightmap_warning, show_marker_dialog};
pub use input::InputState;
pub use menu::render_menu;
pub use options_dialog::show_options_dialog;
pub use properties::render_properties_panel;
pub use status::render_status_bar;
pub use toolbar::render_toolbar;
