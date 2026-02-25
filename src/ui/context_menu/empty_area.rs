//! Empty Area Menu: Rechtsklick auf leeren Bereich.

use super::button_intent;
use crate::app::{AppIntent, EditorTool};

pub fn render_empty_area_menu(ui: &mut egui::Ui, events: &mut Vec<AppIntent>) {
    ui.label("📋 Datei");
    ui.separator();
    button_intent(ui, "📂 Öffnen...", AppIntent::OpenFileRequested, events);
    button_intent(ui, "💾 Speichern", AppIntent::SaveRequested, events);
    button_intent(
        ui,
        "💾 Speichern unter...",
        AppIntent::SaveAsRequested,
        events,
    );

    ui.separator();
    ui.label("🛠 Werkzeug");
    ui.separator();
    button_intent(
        ui,
        "⭘ Auswahl (1)",
        AppIntent::SetEditorToolRequested {
            tool: EditorTool::Select,
        },
        events,
    );
    button_intent(
        ui,
        "⚡ Verbinden (2)",
        AppIntent::SetEditorToolRequested {
            tool: EditorTool::Connect,
        },
        events,
    );
    button_intent(
        ui,
        "➕ Node hinzufügen (3)",
        AppIntent::SetEditorToolRequested {
            tool: EditorTool::AddNode,
        },
        events,
    );

    ui.separator();
    ui.label("📐 Selektion");
    ui.separator();
    button_intent(
        ui,
        "🔍 Alles auswählen",
        AppIntent::SelectAllRequested,
        events,
    );
    button_intent(
        ui,
        "✕ Auswahl löschen",
        AppIntent::ClearSelectionRequested,
        events,
    );

    ui.separator();
    ui.label("🔍 Ansicht");
    ui.separator();
    button_intent(
        ui,
        "📏 Alles einpassen",
        AppIntent::ZoomToFitRequested,
        events,
    );
    button_intent(
        ui,
        "🏠 Kamera zurücksetzen",
        AppIntent::ResetCameraRequested,
        events,
    );

    ui.separator();
    button_intent(ui, "↶ Rückgängig", AppIntent::UndoRequested, events);
    button_intent(ui, "↷ Wiederherstellen", AppIntent::RedoRequested, events);
}
