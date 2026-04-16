//! Application State — zentrale Datenhaltung.

mod app_state;
mod background_layers;
mod dialogs;
mod editor;
mod selection;
mod view;

pub use crate::shared::{
    DedupDialogState, DistanzenState, FloatingMenuKind, FloatingMenuState, GroupSettingsPopupState,
    MarkerDialogState, OverviewOptionsDialogState, OverviewSourceContext, PostLoadDialogState,
    SaveOverviewDialogState, TraceAllFieldsDialogState,
};
pub use app_state::{AppState, Clipboard, GroupEditState};
pub use background_layers::{
    BackgroundLayerCatalog, BackgroundLayerFiles, PendingOverviewBundle, StoredBackgroundLayer,
};
pub use dialogs::{EngineUiState, ZipBrowserState};
pub use editor::{EditorTool, EditorToolState};
pub use selection::SelectionState;
pub use view::ViewState;
