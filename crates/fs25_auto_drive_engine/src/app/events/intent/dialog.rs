macro_rules! dialog_intent_variants {
    () => {
        /// Anwendung beenden
        ExitRequested,
        /// Options-Dialog oeffnen
        OpenOptionsDialogRequested,
        /// Options-Dialog schliessen
        CloseOptionsDialogRequested,
        /// Optionen wurden geaendert (sofortige Anwendung)
        OptionsChanged { options: Box<EditorOptions> },
        /// Optionen auf Standardwerte zuruecksetzen
        ResetOptionsRequested,
        /// Command-Palette oeffnen/schliessen
        CommandPaletteToggled,
        /// Schwebendes Menue an der Mausposition oeffnen/schliessen
        ToggleFloatingMenu {
            kind: crate::app::state::FloatingMenuKind,
        },
    };
}

pub(super) use dialog_intent_variants;

macro_rules! dialog_intent_feature_arms {
    () => {
        Self::ExitRequested
        | Self::OpenOptionsDialogRequested
        | Self::CloseOptionsDialogRequested
        | Self::OptionsChanged { .. }
        | Self::ResetOptionsRequested
        | Self::CommandPaletteToggled
        | Self::ToggleFloatingMenu { .. } => AppEventFeature::Dialog,
    };
}

pub(super) use dialog_intent_feature_arms;