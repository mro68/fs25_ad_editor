//! Schlüssel-Enum für das i18n-System.
//!
//! Jeder Variants-Name entspricht einem UI-String.
//! `match` in den Sprachdateien erzwingt Vollständigkeit bei neuen Keys.

/// Alle übersetzbaren UI-Schlüssel des Editors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum I18nKey {
    // === Allgemein ===
    /// Anwendungstitel
    AppTitle,
    /// Bestätigung
    Ok,
    /// Abbruch
    Cancel,
    /// Übernehmen
    Apply,
    /// Schließen
    Close,
    /// Zurücksetzen
    Reset,
    /// Löschen
    Delete,
    /// Hinzufügen
    Add,
    /// Entfernen
    Remove,
    /// Bezeichnung für Sprachauswahl
    LanguageLabel,

    // === Dialog-Chrome ===
    /// Schaltfläche: Dialog schließen
    DialogClose,
    /// Schaltfläche: Standardwerte wiederherstellen
    DialogDefaults,

    // === Options-Dialog: Abschnitts-Navigation ===
    /// Abschnittstitel "Allgemein"
    OptSectionGeneral,
    /// Abschnittstitel "Nodes"
    OptSectionNodes,
    /// Abschnittstitel "Tools"
    OptSectionTools,
    /// Abschnittstitel "Verbindungen"
    OptSectionConnections,
    /// Abschnittstitel "Verhalten"
    OptSectionBehavior,

    // === Options-Dialog: Abschnitts-Untertitel ===
    /// Untertitel für den Allgemein-Abschnitt
    OptSubtitleGeneral,
    /// Untertitel für den Nodes-Abschnitt
    OptSubtitleNodes,
    /// Untertitel für den Tools-Abschnitt
    OptSubtitleTools,
    /// Untertitel für den Verbindungen-Abschnitt
    OptSubtitleConnections,
    /// Untertitel für den Verhalten-Abschnitt
    OptSubtitleBehavior,

    // === Options-Dialog: Sprache ===
    /// Bezeichnung für das Sprach-Auswahlfeld
    OptLanguageLabel,
    /// Tooltip für das Sprach-Auswahlfeld
    OptLanguageHelp,

    // === Options-Dialog: Fenster-Chrome ===
    /// Fenstertitel des Options-Dialogs
    OptDialogTitle,
    /// Überschrift der Navigations-Seitenleiste
    OptNavHeader,

    // === Options-Dialog: Interne Unterabschnitte (Allgemein) ===
    /// Unterabschnitt-Titel "Selektion"
    OptSubSectionSelection,
    /// Unterabschnitt-Titel "Marker"
    OptSubSectionMarker,
    /// Unterabschnitt-Titel "Kamera"
    OptSubSectionCamera,
    /// Unterabschnitt-Titel "LOD / Mindestgrößen"
    OptSubSectionLod,
    /// Beschreibung des LOD-Unterabschnitts
    OptSubSectionLodDesc,
    /// Unterabschnitt-Titel "Hintergrund"
    OptSubSectionBackground,
    /// Unterabschnitt-Titel "Copy/Paste-Vorschau"
    OptSubSectionCopyPaste,
    /// Unterabschnitt-Titel "Übersichtskarte (Standard-Layer)"
    OptSubSectionOverview,

    // === Options-Dialog: Nodes ===
    /// Label: Node-Größe in Welteinheiten
    OptNodeSizeWorld,
    /// Tooltip: Node-Größe
    OptNodeSizeWorldHelp,
    /// Label: Standard-Knotenfarbe
    OptNodeColorDefault,
    /// Label: SubPrio-Knotenfarbe
    OptNodeColorSubprio,
    /// Label: Farbe selektierter Knoten
    OptNodeColorSelected,
    /// Label: Warnfarbe für Knoten
    OptNodeColorWarning,
    /// Label: Hitbox-Skalierung
    OptHitboxScale,
    /// Tooltip: Hitbox-Skalierung
    OptHitboxScaleHelp,

    // === Options-Dialog: Tools ===
    /// Label: Wertänderungs-Eingabemodus
    OptValueAdjustMode,
    /// ComboBox-Eintrag: LMT ziehen
    OptValueAdjustDrag,
    /// ComboBox-Eintrag: Mausrad
    OptValueAdjustWheel,
    /// Tooltip: Wertänderungs-Eingabemodus
    OptValueAdjustModeHelp,
    /// Label: Snap-Radius
    OptSnapRadius,
    /// Tooltip: Snap-Radius
    OptSnapRadiusHelp,
    /// Label: Mausrad-Schrittweite Distanz
    OptMouseWheelDistStep,
    /// Tooltip: Mausrad-Schrittweite Distanz
    OptMouseWheelDistStepHelp,

    // === Options-Dialog: Selektion (Unterabschnitt) ===
    /// Label: Selektions-Größenfaktor
    OptSelectionSizeFactor,
    /// Tooltip: Selektions-Größenfaktor
    OptSelectionSizeFactorHelp,
    /// Label: Markierungsstil
    OptSelectionStyle,
    /// ComboBox-Eintrag: Ring
    OptSelectionStyleRing,
    /// ComboBox-Eintrag: Farbverlauf
    OptSelectionStyleGradient,
    /// Tooltip: Markierungsstil
    OptSelectionStyleHelp,
    /// Separator-Label: Doppelklick-Segment
    OptDoubleClickSegment,
    /// Checkbox: Bei Kreuzung stoppen
    OptSegmentStopAtJunction,
    /// Tooltip: Bei Kreuzung stoppen
    OptSegmentStopAtJunctionHelp,
    /// Label: Max. Winkel Segmenterkennung
    OptSegmentMaxAngle,
    /// Tooltip: Max. Winkel Segmenterkennung
    OptSegmentMaxAngleHelp,
    /// Schwacher Hinweistext: Segmenterkennung deaktiviert
    OptSegmentDisabled,

    // === Options-Dialog: Verbindungen ===
    /// Label: Breite Hauptstraße
    OptConnectionWidthMain,
    /// Tooltip: Breite Hauptstraße
    OptConnectionWidthMainHelp,
    /// Label: Breite Nebenstraße
    OptConnectionWidthSubprio,
    /// Tooltip: Breite Nebenstraße
    OptConnectionWidthSubprioHelp,
    /// Label: Pfeillänge
    OptArrowLength,
    /// Tooltip: Pfeillänge
    OptArrowLengthHelp,
    /// Label: Pfeilbreite
    OptArrowWidth,
    /// Tooltip: Pfeilbreite
    OptArrowWidthHelp,
    /// Label: Farbe Einbahn vorwärts
    OptConnectionColorRegular,
    /// Label: Farbe Zweirichtungsverkehr
    OptConnectionColorDual,
    /// Label: Farbe Einbahn rückwärts
    OptConnectionColorReverse,

    // === Options-Dialog: Marker ===
    /// Label: Pin-Größe
    OptMarkerSize,
    /// Tooltip: Pin-Größe
    OptMarkerSizeHelp,
    /// Label: Pin-Farbe
    OptMarkerColor,
    /// Label: Umrissstärke Marker
    OptMarkerOutlineWidth,
    /// Tooltip: Umrissstärke Marker
    OptMarkerOutlineWidthHelp,

    // === Options-Dialog: Kamera ===
    /// Label: Minimaler Zoom
    OptCameraZoomMin,
    /// Tooltip: Minimaler Zoom
    OptCameraZoomMinHelp,
    /// Label: Maximaler Zoom
    OptCameraZoomMax,
    /// Tooltip: Maximaler Zoom
    OptCameraZoomMaxHelp,
    /// Label: Zoom-Schritt (Menü)
    OptCameraZoomStep,
    /// Tooltip: Zoom-Schritt (Menü)
    OptCameraZoomStepHelp,
    /// Label: Zoom-Schritt (Scroll)
    OptCameraScrollZoomStep,
    /// Tooltip: Zoom-Schritt (Scroll)
    OptCameraScrollZoomStepHelp,
    /// Label: Zoom-Kompensations-Maximum
    OptZoomCompensationMax,
    /// Tooltip: Zoom-Kompensations-Maximum
    OptZoomCompensationMaxHelp,

    // === Options-Dialog: LOD ===
    /// Label: Mindestgrößen-Gruppe
    OptLodMinSizes,
    /// Label: Nodes (LOD)
    OptLodNodes,
    /// Tooltip: Nodes (LOD)
    OptLodNodesHelp,
    /// Label: Verbindungen (LOD)
    OptLodConnections,
    /// Tooltip: Verbindungen (LOD)
    OptLodConnectionsHelp,
    /// Label: Pfeile (LOD)
    OptLodArrows,
    /// Tooltip: Pfeile (LOD)
    OptLodArrowsHelp,
    /// Label: Marker (LOD)
    OptLodMarkers,
    /// Tooltip: Marker (LOD)
    OptLodMarkersHelp,
    /// Separator-Label: Node-Ausdünnung
    OptLodNodeDecimation,
    /// Label: Mindestabstand Node-Ausdünnung
    OptLodDecimationSpacing,
    /// Tooltip: Mindestabstand Node-Ausdünnung
    OptLodDecimationSpacingHelp,

    // === Options-Dialog: Hintergrund ===
    /// Label: Standard-Deckung Hintergrund
    OptBgOpacity,
    /// Tooltip: Standard-Deckung Hintergrund
    OptBgOpacityHelp,
    /// Label: Deckung bei Min-Zoom
    OptBgOpacityAtMinZoom,
    /// Tooltip: Deckung bei Min-Zoom
    OptBgOpacityAtMinZoomHelp,
    /// Label: Fade-out ab Zoom
    OptBgFadeStartZoom,
    /// Tooltip: Fade-out ab Zoom
    OptBgFadeStartZoomHelp,

    // === Options-Dialog: Übersichtskarte ===
    /// Checkbox: Hillshade-Layer
    OptOverviewHillshade,
    /// Tooltip: Hillshade-Layer
    OptOverviewHillshadeHelp,
    /// Checkbox: Farmland-Grenzen
    OptOverviewFarmlands,
    /// Tooltip: Farmland-Grenzen
    OptOverviewFarmlandsHelp,
    /// Checkbox: Farmland-IDs
    OptOverviewFarmlandIds,
    /// Tooltip: Farmland-IDs
    OptOverviewFarmlandIdsHelp,
    /// Checkbox: POI-Marker
    OptOverviewPois,
    /// Tooltip: POI-Marker
    OptOverviewPoisHelp,
    /// Checkbox: Legende
    OptOverviewLegend,
    /// Tooltip: Legende
    OptOverviewLegendHelp,

    // === Options-Dialog: Verhalten ===
    /// Checkbox: Nach Löschen verbinden
    OptReconnectOnDelete,
    /// Tooltip: Nach Löschen verbinden
    OptReconnectOnDeleteHelp,
    /// Checkbox: Verbindung beim Platzieren teilen
    OptSplitConnectionOnPlace,
    /// Tooltip: Verbindung beim Platzieren teilen
    OptSplitConnectionOnPlaceHelp,

    // === Options-Dialog: Copy/Paste ===
    /// Label: Vorschau-Deckung Copy/Paste
    OptCopyPastePreviewOpacity,
    /// Tooltip: Vorschau-Deckung Copy/Paste
    OptCopyPastePreviewOpacityHelp,
}
