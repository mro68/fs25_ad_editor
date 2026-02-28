use std::collections::HashSet;
use std::sync::Arc;

/// Auswahlbezogener Anwendungszustand
#[derive(Clone, Default)]
pub struct SelectionState {
    /// Menge der aktuell selektierten Node-IDs (Arc für O(1)-Clone in RenderScene)
    pub selected_node_ids: Arc<HashSet<u64>>,
    /// Letzter selektierter Node als Anker für additive Bereichsselektion
    pub selection_anchor_node_id: Option<u64>,
    /// Fokussierter Node für Kontextmenü-Einzelbefehle (RMT auf spezifischen Node)
    pub focused_node_id: Option<u64>,
}

impl SelectionState {
    /// Erstellt einen leeren Selektionszustand.
    pub fn new() -> Self {
        Self {
            selected_node_ids: Arc::new(HashSet::new()),
            selection_anchor_node_id: None,
            focused_node_id: None,
        }
    }

    /// Gibt eine mutable Referenz auf die HashSet zurück (CoW: klont nur wenn nötig).
    ///
    /// Alle Mutationen der Selektion gehen über diese Methode, damit der
    /// Arc-Klon in `RenderScene::build()` O(1) bleibt.
    #[inline]
    pub fn ids_mut(&mut self) -> &mut HashSet<u64> {
        Arc::make_mut(&mut self.selected_node_ids)
    }
}
