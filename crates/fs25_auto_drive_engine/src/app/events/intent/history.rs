macro_rules! history_intent_variants {
    () => {
        /// Undo: Letzte Aktion rueckgaengig machen
        UndoRequested,
        /// Redo: Rueckgaengig gemachte Aktion wiederherstellen
        RedoRequested,
    };
}

pub(super) use history_intent_variants;

macro_rules! history_intent_feature_arms {
    () => {
        Self::UndoRequested | Self::RedoRequested => AppEventFeature::History,
    };
}

pub(super) use history_intent_feature_arms;