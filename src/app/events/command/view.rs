macro_rules! view_command_variants {
    () => {
        /// Kamera auf Standard zuruecksetzen
        ResetCamera,
        /// Stufenweise hineinzoomen
        ZoomIn,
        /// Stufenweise herauszoomen
        ZoomOut,
        /// Viewport-Groesse setzen
        SetViewportSize { size: [f32; 2] },
        /// Kamera um Delta verschieben
        PanCamera { delta: glam::Vec2 },
        /// Kamera zoomen (optional auf Fokuspunkt)
        ZoomCamera {
            factor: f32,
            focus_world: Option<glam::Vec2>,
        },
        /// Kamera auf Node zentrieren (Zoom beibehalten)
        CenterOnNode { node_id: u64 },
        /// Render-Qualitaet setzen
        SetRenderQuality { quality: RenderQuality },
        /// Background-Map laden
        LoadBackgroundMap {
            path: String,
            crop_size: Option<u32>,
        },
        /// Background-Sichtbarkeit umschalten
        ToggleBackgroundVisibility,
        /// Background-Ausdehnung skalieren (Faktor relativ)
        ScaleBackground { factor: f32 },
        /// ZIP-Archiv oeffnen und Bilddateien im Browser anzeigen
        BrowseZipBackground { path: String },
        /// Bilddatei aus ZIP als Background-Map laden
        LoadBackgroundFromZip {
            zip_path: String,
            entry_name: String,
            crop_size: Option<u32>,
        },
        /// Uebersichtskarte generieren (mit Layer-Optionen aus Dialog)
        GenerateOverviewWithOptions,
        /// Background-Map als overview.png im XML-Verzeichnis speichern
        SaveBackgroundAsOverview { path: String },
        /// Alles in den Viewport einpassen (Zoom-to-fit)
        ZoomToFit,
        /// Kamera auf die Bounding Box der Selektion zoomen
        ZoomToSelectionBounds,
    };
}

pub(super) use view_command_variants;

macro_rules! view_command_feature_arms {
    () => {
        Self::ResetCamera
        | Self::ZoomIn
        | Self::ZoomOut
        | Self::SetViewportSize { .. }
        | Self::PanCamera { .. }
        | Self::ZoomCamera { .. }
        | Self::CenterOnNode { .. }
        | Self::SetRenderQuality { .. }
        | Self::LoadBackgroundMap { .. }
        | Self::ToggleBackgroundVisibility
        | Self::ScaleBackground { .. }
        | Self::BrowseZipBackground { .. }
        | Self::LoadBackgroundFromZip { .. }
        | Self::GenerateOverviewWithOptions
        | Self::SaveBackgroundAsOverview { .. }
        | Self::ZoomToFit
        | Self::ZoomToSelectionBounds => AppEventFeature::View,
    };
}

pub(super) use view_command_feature_arms;