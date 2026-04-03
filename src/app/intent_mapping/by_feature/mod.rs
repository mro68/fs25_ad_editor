//! Featurebasiertes Intent-Mapping fuer die Control-Plane.

mod dialog;
mod editing;
mod file_io;
mod group;
mod history;
mod route_tool;
mod selection;
mod view;

use crate::app::events::AppEventFeature;
use crate::app::{AppCommand, AppIntent, AppState};

/// Uebersetzt einen Intent ueber den passenden Feature-Slice in Commands.
pub(super) fn map(state: &AppState, intent: AppIntent) -> Vec<AppCommand> {
    match intent.feature() {
        AppEventFeature::FileIo => file_io::map(state, intent),
        AppEventFeature::View => view::map(state, intent),
        AppEventFeature::Selection => selection::map(state, intent),
        AppEventFeature::Editing => editing::map(state, intent),
        AppEventFeature::RouteTool => route_tool::map(state, intent),
        AppEventFeature::Group => group::map(state, intent),
        AppEventFeature::Dialog => dialog::map(state, intent),
        AppEventFeature::History => history::map(state, intent),
    }
}
