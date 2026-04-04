//! Mapping von UI-Intents auf mutierende App-Commands.

mod by_feature;

use super::{AppCommand, AppIntent, AppState};

/// Uebersetzt einen `AppIntent` in eine Sequenz ausfuehrbarer `AppCommand`s.
pub fn map_intent_to_commands(state: &AppState, intent: AppIntent) -> Vec<AppCommand> {
    by_feature::map(state, intent)
}

#[cfg(test)]
mod tests;
