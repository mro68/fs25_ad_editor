//! Egui-spezifische Adapter-Helfer fuer die gemeinsame Host-Bridge.
//!
//! Dieses Modul mappt einen bewusst kleinen Teil bestehender `AppIntent`s auf
//! die explizite Action-Surface der kanonischen `HostBridgeSession`.
//! Nicht gemappte Intents bleiben im bisherigen egui-Flow und koennen weiterhin
//! direkt ueber `AppController` verarbeitet werden.

use anyhow::Result;
use fs25_auto_drive_host_bridge::{
    HostActiveTool, HostBridgeSession, HostSessionAction,
};

use crate::app::{AppIntent, EditorTool};

fn map_active_tool(tool: EditorTool) -> HostActiveTool {
    match tool {
        EditorTool::Select => HostActiveTool::Select,
        EditorTool::Connect => HostActiveTool::Connect,
        EditorTool::AddNode => HostActiveTool::AddNode,
        EditorTool::Route => HostActiveTool::Route,
    }
}

/// Mappt einen egui-`AppIntent` auf eine explizite Host-Bridge-Action.
///
/// Rueckgabewert `None` bedeutet, dass der Intent aktuell ausserhalb der
/// schlanken Adapter-Surface liegt und weiterhin ueber den bestehenden
/// Controller-Flow verarbeitet werden soll.
pub fn map_intent_to_host_action(intent: &AppIntent) -> Option<HostSessionAction> {
    match intent {
        AppIntent::CommandPaletteToggled => Some(HostSessionAction::ToggleCommandPalette),
        AppIntent::SetEditorToolRequested { tool } => Some(HostSessionAction::SetEditorTool {
            tool: map_active_tool(*tool),
        }),
        AppIntent::OpenOptionsDialogRequested => Some(HostSessionAction::OpenOptionsDialog),
        AppIntent::CloseOptionsDialogRequested => Some(HostSessionAction::CloseOptionsDialog),
        AppIntent::UndoRequested => Some(HostSessionAction::Undo),
        AppIntent::RedoRequested => Some(HostSessionAction::Redo),
        _ => None,
    }
}

/// Wendet einen gemappten Intent direkt auf eine `HostBridgeSession` an.
///
/// Gibt `Ok(true)` zurueck, wenn der Intent gemappt und angewendet wurde.
/// `Ok(false)` bedeutet, dass kein Mapping existiert.
pub fn apply_mapped_intent(session: &mut HostBridgeSession, intent: &AppIntent) -> Result<bool> {
    let Some(action) = map_intent_to_host_action(intent) else {
        return Ok(false);
    };

    session.apply_action(action)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use fs25_auto_drive_host_bridge::HostSessionAction;

    use crate::app::{AppIntent, EditorTool};

    use super::map_intent_to_host_action;

    #[test]
    fn maps_command_palette_toggle_to_host_action() {
        let intent = AppIntent::CommandPaletteToggled;
        let action = map_intent_to_host_action(&intent);

        assert!(matches!(
            action,
            Some(HostSessionAction::ToggleCommandPalette)
        ));
    }

    #[test]
    fn maps_set_editor_tool_to_host_action() {
        let intent = AppIntent::SetEditorToolRequested {
            tool: EditorTool::Route,
        };
        let action = map_intent_to_host_action(&intent);

        assert!(matches!(
            action,
            Some(HostSessionAction::SetEditorTool {
                tool: fs25_auto_drive_host_bridge::HostActiveTool::Route
            })
        ));
    }

    #[test]
    fn leaves_unmapped_intents_for_existing_controller_flow() {
        let intent = AppIntent::OpenFileRequested;
        let action = map_intent_to_host_action(&intent);

        assert!(action.is_none());
    }
}
