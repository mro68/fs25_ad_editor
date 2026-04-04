//! UI/System-Intents als nicht-mutierende Eingabeebene.

mod definition;

pub use definition::AppIntent;

#[cfg(test)]
mod tests {
    use super::AppIntent;
    use crate::app::events::AppEventFeature;
    use crate::app::ui_contract::{ParkingPanelAction, RouteToolPanelAction};

    #[test]
    fn classifies_editing_group_and_dialog_intents() {
        assert_eq!(
            AppIntent::PasteCancelled.feature(),
            AppEventFeature::Editing
        );
        assert_eq!(
            AppIntent::GroupEditToolRequested { record_id: 7 }.feature(),
            AppEventFeature::Group
        );
        assert_eq!(
            AppIntent::CommandPaletteToggled.feature(),
            AppEventFeature::Dialog
        );
    }

    #[test]
    fn classifies_view_and_route_tool_intents() {
        assert_eq!(
            AppIntent::GenerateOverviewRequested.feature(),
            AppEventFeature::View
        );
        assert_eq!(
            AppIntent::RouteToolPanelActionRequested {
                action: RouteToolPanelAction::Parking(ParkingPanelAction::SetNumRows(3)),
            }
            .feature(),
            AppEventFeature::RouteTool
        );
    }
}
