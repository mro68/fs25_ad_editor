//! Application State — zentrale Datenhaltung.

mod app_state;
mod dialogs;
mod editor;
mod selection;
mod view;

pub use app_state::{AppState, Clipboard, GroupEditState};
pub use dialogs::{
    DedupDialogState, DistanzenState, EngineUiState, FloatingMenuKind, FloatingMenuState,
    GroupSettingsPopupState, MarkerDialogState, OverviewOptionsDialogState, PostLoadDialogState,
    SaveOverviewDialogState, TraceAllFieldsDialogState, ZipBrowserState,
};
pub use editor::{EditorTool, EditorToolState};
pub use selection::SelectionState;
pub use view::ViewState;
