macro_rules! file_io_command_variants {
    () => {
        /// Datei-Oeffnen-Dialog anfordern
        RequestOpenFileDialog,
        /// Datei-Speichern-Dialog anfordern
        RequestSaveFileDialog,
        /// Heightmap entfernen
        ClearHeightmap,
        /// Speichern nach Heightmap-Warnung bestaetigen
        ConfirmAndSaveFile,
        /// XML-Datei laden
        LoadFile { path: String },
        /// Datei speichern (None = aktueller Pfad, Some(p) = neuer Pfad)
        SaveFile { path: Option<String> },
        /// Heightmap setzen
        SetHeightmap { path: String },
        /// Duplikat-Bereinigung durchfuehren
        DeduplicateNodes,
    };
}

pub(super) use file_io_command_variants;

macro_rules! file_io_command_feature_arms {
    () => {
        Self::RequestOpenFileDialog
        | Self::RequestSaveFileDialog
        | Self::ConfirmAndSaveFile
        | Self::LoadFile { .. }
        | Self::SaveFile { .. }
        | Self::ClearHeightmap
        | Self::SetHeightmap { .. }
        | Self::DeduplicateNodes => AppEventFeature::FileIo,
    };
}

pub(super) use file_io_command_feature_arms;