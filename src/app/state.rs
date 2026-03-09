//! Application State — zentrale Datenhaltung.

mod app_state;
mod dialogs;
mod editor;
mod selection;
mod view;

pub use app_state::{AppState, Clipboard};
pub use dialogs::{
    DedupDialogState, DistanzenState, FloatingMenuKind, FloatingMenuState, MarkerDialogState,
    OverviewOptionsDialogState, PostLoadDialogState, SaveOverviewDialogState, UiState,
    ZipBrowserState,
};
pub use editor::{EditorTool, EditorToolState};
pub use selection::SelectionState;
pub use view::ViewState;
