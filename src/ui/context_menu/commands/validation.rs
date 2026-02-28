//! Validierung und Filterung von Menü-Einträgen.
//!
//! Prüft Preconditions und entfernt überflüssige Separatoren/Labels.

use super::preconditions::{Precondition, PreconditionContext};
use super::{CommandId, IntentContext, MenuCatalog, MenuEntry};
use crate::app::AppIntent;

/// Prüft ob alle Preconditions eines Menu-Eintrags erfüllt sind.
pub(crate) fn all_preconditions_valid(
    preconditions: &[Precondition],
    ctx: &PreconditionContext,
) -> bool {
    preconditions.iter().all(|p| p.is_valid(ctx))
}

/// Ergebnis der Validierung: Sichtbare Einträge mit ihrem Intent.
#[derive(Debug)]
pub enum ValidatedEntry {
    /// Label (immer sichtbar)
    Label(String),
    /// Trennlinie (wird nur angezeigt wenn umgebende Commands sichtbar sind)
    Separator,
    /// Gültiger Befehl mit fertigem Intent
    Command {
        #[allow(dead_code)]
        id: CommandId,
        label: String,
        intent: Box<AppIntent>,
    },
}

/// Validiert einen MenuCatalog und gibt nur die sichtbaren Einträge zurück.
///
/// Separatoren werden intelligent gefiltert: Doppelte Separatoren und
/// Separatoren am Anfang/Ende werden entfernt.
pub fn validate_entries(
    catalog: &MenuCatalog,
    precondition_ctx: &PreconditionContext,
    intent_ctx: &IntentContext,
) -> Vec<ValidatedEntry> {
    let mut raw: Vec<ValidatedEntry> = Vec::new();

    for entry in &catalog.entries {
        match entry {
            MenuEntry::Label(text) => {
                raw.push(ValidatedEntry::Label(text.clone()));
            }
            MenuEntry::Separator => {
                raw.push(ValidatedEntry::Separator);
            }
            MenuEntry::Command {
                id,
                label,
                preconditions,
            } => {
                if all_preconditions_valid(preconditions, precondition_ctx) {
                    raw.push(ValidatedEntry::Command {
                        id: *id,
                        label: label.clone(),
                        intent: Box::new(id.to_intent(intent_ctx)),
                    });
                }
            }
        }
    }

    // Separatoren bereinigen: keine doppelten, keine am Anfang/Ende,
    // keine direkt nach Label ohne folgendem Command
    cleanup_separators(raw)
}

/// Entfernt überflüssige Separatoren und Labels ohne folgende Commands.
pub(crate) fn cleanup_separators(entries: Vec<ValidatedEntry>) -> Vec<ValidatedEntry> {
    let mut result: Vec<ValidatedEntry> = Vec::new();

    for entry in entries {
        match &entry {
            ValidatedEntry::Separator => {
                // Separator nur wenn vorheriger Eintrag kein Separator ist und es vorherige Einträge gibt
                if !result.is_empty() && !matches!(result.last(), Some(ValidatedEntry::Separator)) {
                    result.push(entry);
                }
            }
            _ => {
                result.push(entry);
            }
        }
    }

    // Trailing Separator entfernen
    if matches!(result.last(), Some(ValidatedEntry::Separator)) {
        result.pop();
    }

    // Labels ohne nachfolgende Commands entfernen (Sektion ohne Einträge)
    remove_orphaned_labels(result)
}

/// Entfernt Labels die nicht von mindestens einem Command gefolgt werden
/// (bevor der nächste Separator oder das Ende kommt).
pub(crate) fn remove_orphaned_labels(entries: Vec<ValidatedEntry>) -> Vec<ValidatedEntry> {
    let len = entries.len();
    // Markiere welche Indizes behalten werden
    let mut keep = vec![true; len];

    for i in 0..len {
        if matches!(&entries[i], ValidatedEntry::Label(_)) {
            // Prüfe ob nach diesem Label (bis zum nächsten Separator/Label/Ende) ein Command kommt
            let has_following_command = entries[i + 1..]
                .iter()
                .take_while(|e| !matches!(e, ValidatedEntry::Separator | ValidatedEntry::Label(_)))
                .any(|e| matches!(e, ValidatedEntry::Command { .. }));

            if !has_following_command {
                keep[i] = false;
            }
        }
    }

    entries
        .into_iter()
        .enumerate()
        .filter(|(i, _)| keep[*i])
        .map(|(_, e)| e)
        .collect()
}
