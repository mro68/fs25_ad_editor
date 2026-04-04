macro_rules! view_intent_variants {
    () => {
        /// Background-Map-Auswahldialog oeffnen
        BackgroundMapSelectionRequested,
        /// Kamera auf Standard zuruecksetzen
        ResetCameraRequested,
        /// Stufenweise hineinzoomen
        ZoomInRequested,
        /// Stufenweise herauszoomen
        ZoomOutRequested,
        /// Viewport-Groesse hat sich geaendert
        ViewportResized { size: [f32; 2] },
        /// Kamera um Delta verschieben (Welt-Einheiten)
        CameraPan { delta: glam::Vec2 },
        /// Kamera zoomen (optional auf einen Fokuspunkt)
        CameraZoom {
            factor: f32,
            focus_world: Option<glam::Vec2>,
        },
        /// Kamera auf einen bestimmten Node zentrieren (Zoom beibehalten)
        CenterOnNodeRequested { node_id: u64 },
        /// Render-Qualitaetsstufe aendern
        RenderQualityChanged { quality: RenderQuality },
        /// Background-Map auswaehlen
        BackgroundMapSelected {
            path: String,
            crop_size: Option<u32>,
        },
        /// Background-Sichtbarkeit umschalten
        ToggleBackgroundVisibility,
        /// Background-Ausdehnung skalieren (Faktor relativ, z.B. 2.0 = verdoppeln)
        ScaleBackground { factor: f32 },
        /// ZIP-Datei wurde als Background-Map gewaehlt → Browser oeffnen
        ZipBackgroundBrowseRequested { path: String },
        /// Bilddatei aus ZIP-Browser gewaehlt
        ZipBackgroundFileSelected {
            zip_path: String,
            entry_name: String,
        },
        /// ZIP-Browser geschlossen (ohne Auswahl)
        ZipBrowserCancelled,
        /// Uebersichtskarte aus Map-Mod-ZIP generieren (oeffnet Dateidialog)
        GenerateOverviewRequested,
        /// ZIP fuer Uebersichtskarte gewaehlt → Options-Dialog anzeigen
        GenerateOverviewFromZip { path: String },
        /// Uebersichtskarten-Options-Dialog bestaetigt (generieren)
        OverviewOptionsConfirmed,
        /// Uebersichtskarten-Options-Dialog abgebrochen
        OverviewOptionsCancelled,
        /// Post-Load-Dialog: Uebersichtskarte generieren (ZIP-Pfad ausgewaehlt)
        PostLoadGenerateOverview { zip_path: String },
        /// Post-Load-Dialog: geschlossen ohne Aktion
        PostLoadDialogDismissed,
        /// Benutzer hat bestaetigt: Background als overview.png speichern
        SaveBackgroundAsOverviewConfirmed,
        /// Benutzer hat abgelehnt: overview.png nicht speichern
        SaveBackgroundAsOverviewDismissed,
        /// Alles in den Viewport einpassen (Zoom-to-fit)
        ZoomToFitRequested,
        /// Viewport auf die Grenzen der aktuellen Selektion einpassen
        ZoomToSelectionBoundsRequested,
    };
}

pub(super) use view_intent_variants;

macro_rules! view_intent_feature_arms {
    () => {
        Self::BackgroundMapSelectionRequested
        | Self::ResetCameraRequested
        | Self::ZoomInRequested
        | Self::ZoomOutRequested
        | Self::ViewportResized { .. }
        | Self::CameraPan { .. }
        | Self::CameraZoom { .. }
        | Self::CenterOnNodeRequested { .. }
        | Self::RenderQualityChanged { .. }
        | Self::BackgroundMapSelected { .. }
        | Self::ToggleBackgroundVisibility
        | Self::ScaleBackground { .. }
        | Self::ZipBackgroundBrowseRequested { .. }
        | Self::ZipBackgroundFileSelected { .. }
        | Self::ZipBrowserCancelled
        | Self::GenerateOverviewRequested
        | Self::GenerateOverviewFromZip { .. }
        | Self::OverviewOptionsConfirmed
        | Self::OverviewOptionsCancelled
        | Self::PostLoadGenerateOverview { .. }
        | Self::PostLoadDialogDismissed
        | Self::SaveBackgroundAsOverviewConfirmed
        | Self::SaveBackgroundAsOverviewDismissed
        | Self::ZoomToFitRequested
        | Self::ZoomToSelectionBoundsRequested => AppEventFeature::View,
    };
}

pub(super) use view_intent_feature_arms;