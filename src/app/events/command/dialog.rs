macro_rules! dialog_command_variants {
    () => {
        /// Anwendung beenden
        RequestExit,
        /// Heightmap-Dialog anfordern
        RequestHeightmapDialog,
        /// Background-Map-Dialog anfordern
        RequestBackgroundMapDialog,
        /// Heightmap-Warnung schliessen
        DismissHeightmapWarning,
        /// Marker-Dialog schliessen
        CloseMarkerDialog,
        /// Duplikat-Dialog schliessen (ohne Bereinigung)
        DismissDeduplicateDialog,
        /// Options-Dialog oeffnen
        OpenOptionsDialog,
        /// Options-Dialog schliessen
        CloseOptionsDialog,
        /// Optionen anwenden und speichern
        ApplyOptions { options: Box<EditorOptions> },
        /// Optionen auf Standardwerte zuruecksetzen
        ResetOptions,
        /// Command-Palette ein-/ausblenden
        ToggleCommandPalette,
        /// ZIP-Browser-Dialog schliessen
        CloseZipBrowser,
        /// Uebersichtskarten-ZIP-Dialog anfordern
        RequestOverviewDialog,
        /// Uebersichtskarten-Options-Dialog mit ZIP-Pfad oeffnen
        OpenOverviewOptionsDialog { path: String },
        /// Uebersichtskarten-Options-Dialog schliessen
        CloseOverviewOptionsDialog,
        /// Post-Load-Dialog schliessen
        DismissPostLoadDialog,
        /// overview.png-Speichern-Dialog schliessen
        DismissSaveOverviewDialog,
        /// Einstellungsdialog "Alle Felder nachzeichnen" oeffnen
        OpenTraceAllFieldsDialog,
        /// Einstellungsdialog "Alle Felder nachzeichnen" schliessen (Abbruch)
        CloseTraceAllFieldsDialog,
        /// Curseplay-Import-Dateidialog anfordern
        RequestCurseplayImportDialog,
        /// Curseplay-Export-Dateidialog anfordern
        RequestCurseplayExportDialog,
    };
}

pub(super) use dialog_command_variants;

macro_rules! dialog_command_feature_arms {
    () => {
        Self::RequestExit
        | Self::RequestHeightmapDialog
        | Self::RequestBackgroundMapDialog
        | Self::DismissHeightmapWarning
        | Self::CloseMarkerDialog
        | Self::OpenOptionsDialog
        | Self::CloseOptionsDialog
        | Self::ApplyOptions { .. }
        | Self::ResetOptions
        | Self::ToggleCommandPalette
        | Self::DismissDeduplicateDialog
        | Self::CloseZipBrowser
        | Self::RequestOverviewDialog
        | Self::OpenOverviewOptionsDialog { .. }
        | Self::CloseOverviewOptionsDialog
        | Self::DismissPostLoadDialog
        | Self::DismissSaveOverviewDialog
        | Self::OpenTraceAllFieldsDialog
        | Self::CloseTraceAllFieldsDialog
        | Self::RequestCurseplayImportDialog
        | Self::RequestCurseplayExportDialog => AppEventFeature::Dialog,
    };
}

pub(super) use dialog_command_feature_arms;