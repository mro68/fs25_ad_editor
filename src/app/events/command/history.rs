macro_rules! history_command_variants {
    () => {
        /// Undo: Letzte Aktion rueckgaengig machen
        Undo,
        /// Redo: Rueckgaengig gemachte Aktion wiederherstellen
        Redo,
    };
}

pub(super) use history_command_variants;

macro_rules! history_command_feature_arms {
    () => {
        Self::Undo | Self::Redo => AppEventFeature::History,
    };
}

pub(super) use history_command_feature_arms;