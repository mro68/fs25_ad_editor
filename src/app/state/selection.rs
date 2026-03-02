use indexmap::IndexSet;
use std::sync::Arc;

/// Auswahlbezogener Anwendungszustand
#[derive(Clone, Default)]
pub struct SelectionState {
    /// Geordnete Menge der aktuell selektierten Node-IDs (Arc für O(1)-Clone in RenderScene).
    /// Die Einfüge-Reihenfolge entspricht der Klick-Reihenfolge — wichtig für gerichtete
    /// Operationen wie "Verbinden" (erster Klick = from, zweiter Klick = to).
    pub selected_node_ids: Arc<IndexSet<u64>>,
    /// Letzter selektierter Node als Anker für additive Bereichsselektion
    pub selection_anchor_node_id: Option<u64>,
}

impl SelectionState {
    /// Erstellt einen leeren Selektionszustand.
    pub fn new() -> Self {
        Self {
            selected_node_ids: Arc::new(IndexSet::new()),
            selection_anchor_node_id: None,
        }
    }

    /// Gibt eine mutable Referenz auf die IndexSet zurück (CoW: klont nur wenn nötig).
    ///
    /// Alle Mutationen der Selektion gehen über diese Methode, damit der
    /// Arc-Klon in `RenderScene::build()` O(1) bleibt.
    #[inline]
    pub fn ids_mut(&mut self) -> &mut IndexSet<u64> {
        Arc::make_mut(&mut self.selected_node_ids)
    }
}
