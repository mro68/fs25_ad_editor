//! Precondition-System fuer Context-Menu-Befehle.
//!
//! Pruefbare Vorbedingungen als Enum (kein dyn Trait, performant).

use crate::app::RoadMap;
use indexmap::IndexSet;

/// Pruefbare Vorbedingung fuer einen Context-Menu-Befehl.
#[derive(Debug, Clone, Copy)]
pub enum Precondition {
    /// Node existiert noch in der RoadMap
    NodeExists(u64),
    /// Node hat einen Marker
    HasMarker(u64),
    /// Node hat keinen Marker
    HasNoMarker(u64),
    /// Genau 2 Nodes selektiert
    ExactlyTwoSelected,
    /// Genau 2 Nodes selektiert UND keine Verbindung dazwischen
    TwoSelectedUnconnected,
    /// Es gibt Verbindungen zwischen selektierten Nodes
    HasConnectionsBetweenSelected,
    /// Streckenteilung ist aktiv
    StreckenteilungActive(bool),
    /// Selektion bildet eine zusammenhaengende Kette (fuer Streckenteilung)
    IsResampleableChain,
}

/// Kontext fuer die Precondition-Auswertung — alle noetigen Daten aus dem aktuellen State.
pub struct PreconditionContext<'a> {
    pub road_map: &'a RoadMap,
    pub selected_node_ids: &'a IndexSet<u64>,
    /// Ob die Streckenteilung gerade aktiv ist
    pub distanzen_active: bool,
}

impl Precondition {
    /// Prueft ob die Vorbedingung im gegebenen Kontext erfuellt ist.
    pub fn is_valid(&self, ctx: &PreconditionContext) -> bool {
        match self {
            Self::NodeExists(id) => ctx.road_map.nodes.contains_key(id),

            Self::HasMarker(id) => ctx.road_map.has_marker(*id),

            Self::HasNoMarker(id) => !ctx.road_map.has_marker(*id),

            Self::ExactlyTwoSelected => ctx.selected_node_ids.len() == 2,

            Self::TwoSelectedUnconnected => {
                if ctx.selected_node_ids.len() != 2 {
                    return false;
                }
                let ids: Vec<u64> = ctx.selected_node_ids.iter().copied().collect();
                let (a, b) = (ids[0], ids[1]);
                // Keine Verbindung in beide Richtungen
                !ctx.road_map.has_connection(a, b) && !ctx.road_map.has_connection(b, a)
            }

            Self::HasConnectionsBetweenSelected => ctx
                .road_map
                .connections_between_ids(ctx.selected_node_ids)
                .next()
                .is_some(),

            Self::StreckenteilungActive(expected) => ctx.distanzen_active == *expected,

            Self::IsResampleableChain => ctx.road_map.is_resampleable_chain(ctx.selected_node_ids),
        }
    }
}
