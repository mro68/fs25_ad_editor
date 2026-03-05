//! Validierung und Filterung von Menue-Eintraegen.
//!
//! Prueft Preconditions und entfernt ueberfluessige Separatoren/Labels.

use super::preconditions::{Precondition, PreconditionContext};
use super::{CommandId, IntentContext, MenuCatalog, MenuEntry};
use crate::app::AppIntent;

/// Prueft ob alle Preconditions eines Menu-Eintrags erfuellt sind.
pub(crate) fn all_preconditions_valid(
    preconditions: &[Precondition],
    ctx: &PreconditionContext,
) -> bool {
    preconditions.iter().all(|p| p.is_valid(ctx))
}

/// Ergebnis der Validierung: Sichtbare Eintraege mit ihrem Intent.
#[derive(Debug, Clone)]
pub enum ValidatedEntry {
    /// Label (immer sichtbar)
    Label(String),
    /// Trennlinie (wird nur angezeigt wenn umgebende Commands sichtbar sind)
    Separator,
    /// Gueltiger Befehl mit fertigem Intent
    Command {
        id: CommandId,
        label: String,
        intent: Box<AppIntent>,
    },
    /// Einklappbares Untermenue (nur sichtbar wenn ≥1 Command enthalten)
    Submenu {
        label: String,
        entries: Vec<ValidatedEntry>,
    },
}

/// Validiert einen MenuCatalog und gibt nur die sichtbaren Eintraege zurueck.
///
/// Separatoren werden intelligent gefiltert: Doppelte Separatoren und
/// Separatoren am Anfang/Ende werden entfernt.
/// Submenues ohne sichtbare Commands werden komplett ausgeblendet.
pub fn validate_entries(
    catalog: &MenuCatalog,
    precondition_ctx: &PreconditionContext,
    intent_ctx: &IntentContext,
) -> Vec<ValidatedEntry> {
    let raw = validate_entries_recursive(&catalog.entries, precondition_ctx, intent_ctx);
    cleanup_separators(&raw)
}

/// Rekursive Validierung fuer verschachtelte Menue-Eintraege.
fn validate_entries_recursive(
    entries: &[MenuEntry],
    precondition_ctx: &PreconditionContext,
    intent_ctx: &IntentContext,
) -> Vec<ValidatedEntry> {
    let mut raw: Vec<ValidatedEntry> = Vec::new();

    for entry in entries {
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
            MenuEntry::Submenu {
                label,
                entries: sub_entries,
            } => {
                let validated_children =
                    validate_entries_recursive(sub_entries, precondition_ctx, intent_ctx);
                // Submenu nur anzeigen wenn mindestens ein Command darin sichtbar ist
                let has_commands = validated_children
                    .iter()
                    .any(|e| matches!(e, ValidatedEntry::Command { .. }));
                if has_commands {
                    raw.push(ValidatedEntry::Submenu {
                        label: label.clone(),
                        entries: cleanup_separators(&validated_children),
                    });
                }
            }
        }
    }

    raw
}

/// Entfernt ueberfluessige Separatoren und Labels ohne folgende Commands.
pub(crate) fn cleanup_separators(entries: &[ValidatedEntry]) -> Vec<ValidatedEntry> {
    let mut result: Vec<ValidatedEntry> = Vec::new();

    for entry in entries {
        match entry {
            ValidatedEntry::Separator => {
                // Separator nur wenn vorheriger Eintrag kein Separator ist und es vorherige Eintraege gibt
                if !result.is_empty() && !matches!(result.last(), Some(ValidatedEntry::Separator)) {
                    result.push(entry.clone());
                }
            }
            _ => {
                result.push(entry.clone());
            }
        }
    }

    // Trailing Separator entfernen
    if matches!(result.last(), Some(ValidatedEntry::Separator)) {
        result.pop();
    }

    // Labels ohne nachfolgende Commands entfernen (Sektion ohne Eintraege)
    remove_orphaned_labels(&result)
}

/// Entfernt Labels die nicht von mindestens einem Command gefolgt werden
/// (bevor der naechste Separator oder das Ende kommt).
pub(crate) fn remove_orphaned_labels(entries: &[ValidatedEntry]) -> Vec<ValidatedEntry> {
    let len = entries.len();
    // Markiere welche Indizes behalten werden
    let mut keep = vec![true; len];

    for i in 0..len {
        if matches!(&entries[i], ValidatedEntry::Label(_)) {
            // Pruefe ob nach diesem Label (bis zum naechsten Separator/Label/Ende) ein Command kommt
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
        .iter()
        .cloned()
        .enumerate()
        .filter(|(i, _)| keep[*i])
        .map(|(_, e)| e)
        .collect()
}
