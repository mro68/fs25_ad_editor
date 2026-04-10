//! Mutierende App-Commands fuer den zentralen Controller-Dispatch.

mod definition;

pub use definition::AppCommand;

#[cfg(test)]
mod tests {
    use super::AppCommand;
    use crate::app::events::AppEventFeature;
    use crate::app::ui_contract::{ParkingPanelAction, RouteToolPanelAction};

    #[test]
    fn classifies_dialog_group_and_editing_commands() {
        assert_eq!(
            AppCommand::OpenOverviewSourceDialog.feature(),
            AppEventFeature::Dialog
        );
        assert_eq!(
            AppCommand::OpenDissolveConfirmDialog { segment_id: 1 }.feature(),
            AppEventFeature::Group
        );
        assert_eq!(AppCommand::ConfirmPaste.feature(), AppEventFeature::Editing);
    }

    #[test]
    fn classifies_view_and_route_tool_commands() {
        assert_eq!(
            AppCommand::GenerateOverviewWithOptions.feature(),
            AppEventFeature::View
        );
        assert_eq!(
            AppCommand::RouteToolPanelAction {
                action: RouteToolPanelAction::Parking(ParkingPanelAction::SetNumRows(3)),
            }
            .feature(),
            AppEventFeature::RouteTool
        );
    }
}
