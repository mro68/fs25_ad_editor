macro_rules! file_io_intent_variants {
    () => {
        /// Datei oeffnen (zeigt Dateidialog)
        OpenFileRequested,
        /// Datei speichern (unter aktuellem Pfad oder mit Dialog)
        SaveRequested,
        /// Datei unter neuem Pfad speichern
        SaveAsRequested,
        /// Heightmap-Auswahldialog oeffnen
        HeightmapSelectionRequested,
        /// Heightmap entfernen
        HeightmapCleared,
        /// Heightmap-Warnung bestaetigt (Speichern fortsetzen)
        HeightmapWarningConfirmed,
        /// Heightmap-Warnung abgebrochen
        HeightmapWarningCancelled,
        /// Datei wurde im Dialog ausgewaehlt (Laden)
        FileSelected { path: String },
        /// Speicherpfad wurde im Dialog ausgewaehlt
        SaveFilePathSelected { path: String },
        /// Heightmap-Datei wurde im Dialog ausgewaehlt
        HeightmapSelected { path: String },
        /// Duplikat-Bereinigung bestaetigt
        DeduplicateConfirmed,
        /// Duplikat-Bereinigung abgelehnt
        DeduplicateCancelled,
    };
}

pub(super) use file_io_intent_variants;

macro_rules! file_io_intent_feature_arms {
    () => {
        Self::OpenFileRequested
        | Self::SaveRequested
        | Self::SaveAsRequested
        | Self::HeightmapSelectionRequested
        | Self::HeightmapCleared
        | Self::HeightmapWarningConfirmed
        | Self::HeightmapWarningCancelled
        | Self::FileSelected { .. }
        | Self::SaveFilePathSelected { .. }
        | Self::HeightmapSelected { .. }
        | Self::DeduplicateConfirmed
        | Self::DeduplicateCancelled => AppEventFeature::FileIo,
    };
}

pub(super) use file_io_intent_feature_arms;