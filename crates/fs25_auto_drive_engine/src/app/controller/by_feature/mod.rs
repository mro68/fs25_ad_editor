//! Featurebasierter Command-Dispatch fuer den AppController.

mod dialog;
mod editing;
mod file_io;
mod group;
mod history;
mod route_tool;
mod selection;
mod view;

use crate::app::events::AppEventFeature;
use crate::app::{AppCommand, AppState};

/// Dispatcht einen Command in den passenden Feature-Slice.
pub(super) fn handle(state: &mut AppState, command: AppCommand) -> anyhow::Result<()> {
    match command.feature() {
        AppEventFeature::FileIo => file_io::handle(state, command),
        AppEventFeature::View => view::handle(state, command),
        AppEventFeature::Selection => selection::handle(state, command),
        AppEventFeature::Editing => editing::handle(state, command),
        AppEventFeature::RouteTool => route_tool::handle(state, command),
        AppEventFeature::Group => group::handle(state, command),
        AppEventFeature::Dialog => dialog::handle(state, command),
        AppEventFeature::History => history::handle(state, command),
    }
}
