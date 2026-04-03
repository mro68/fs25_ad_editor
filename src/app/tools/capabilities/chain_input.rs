//! Capability fuer Tools, die eine geordnete Kette als Eingabe benoetigen.

use glam::Vec2;

/// Geordnete Kette als Tool-Eingabe.
#[derive(Debug, Clone)]
pub struct OrderedNodeChain {
    /// Weltpositionen der geordneten Kette.
    pub positions: Vec<Vec2>,
    /// Start-Node-ID der Kette.
    pub start_id: u64,
    /// End-Node-ID der Kette.
    pub end_id: u64,
    /// Innere Node-IDs der Kette ohne Start und Ende.
    pub inner_ids: Vec<u64>,
}

/// Optionale Capability fuer Tools mit Ketteneingabe.
pub trait RouteToolChainInput {
    /// Laedt eine geordnete Kette als Tool-Eingabe.
    fn load_chain(&mut self, chain: OrderedNodeChain);
}
