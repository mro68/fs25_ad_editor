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

    // === Menü: Datei ===
    /// Menüeintrag "Datei"
    MenuFile,
    /// Menüeintrag "Öffnen…"
    MenuOpen,
    /// Menüeintrag "Speichern"
    MenuSave,
    /// Menüeintrag "Speichern unter…"
    MenuSaveAs,
    /// Menüeintrag "Höhenkarte auswählen…"
    MenuSelectHeightmap,
    /// Menüeintrag "Höhenkarte ändern…"
    MenuChangeHeightmap,
    /// Menüeintrag "Höhenkarte entfernen"
    MenuClearHeightmap,
    /// Menüeintrag "Übersichtskarte generieren…"
    MenuGenerateOverview,
    /// Menüeintrag "Beenden"
    MenuExit,

    // === Menü: Bearbeiten ===
    /// Menüeintrag "Bearbeiten"
    MenuEdit,
    /// Menüeintrag "Rückgängig (Ctrl+Z)"
    MenuUndo,
    /// Menüeintrag "Wiederherstellen (Ctrl+Y)"
    MenuRedo,
    /// Menüeintrag "Kopieren (Ctrl+C)"
    MenuCopy,
    /// Menüeintrag "Einfügen (Ctrl+V)"
    MenuPaste,
    /// Menüeintrag "Optionen…"
    MenuOptions,

    // === Menü: Ansicht ===
    /// Menüeintrag "Ansicht"
    MenuView,
    /// Menüeintrag "Kamera zurücksetzen"
    MenuResetCamera,
    /// Menüeintrag "Vergrößern"
    MenuZoomIn,
    /// Menüeintrag "Verkleinern"
    MenuZoomOut,
    /// Menüeintrag "Hintergrund laden…"
    MenuLoadBackground,
    /// Menüeintrag "Hintergrund ändern…"
    MenuChangeBackground,
    /// Untermenü "Renderqualität"
    MenuRenderQuality,
    /// Qualitätsstufe "Niedrig"
    MenuQualityLow,
    /// Qualitätsstufe "Mittel"
    MenuQualityMedium,
    /// Qualitätsstufe "Hoch"
    MenuQualityHigh,

    // === Menü: Extras ===
    /// Menüeintrag "Extras"
    MenuExtras,
    /// Menüeintrag "Feld erkennen"
    MenuDetectField,
    /// Menüeintrag "Alle Felder nachzeichnen"
    MenuTraceAllFields,
    /// Disabled-Tooltip: Hintergrund mit Feldgrenzen zuerst laden
    MenuExtrasNeedBackground,
    /// Hover-Tooltip: Alle Felder nachzeichnen (Beschreibung)
    MenuTraceAllFieldsHelp,

    // === Menü: Hilfe ===
    /// Menüeintrag "Hilfe"
    MenuHelp,
    /// Menüeintrag "Über"
    MenuAbout,

    // === Status-Bar ===
    /// Status: Keine Datei geladen
    StatusNoFile,
    /// Status-Label "Knoten"
    StatusNodes,
    /// Status-Label "Verbindungen"
    StatusConnections,
    /// Status-Label "Marker"
    StatusMarkers,
    /// Status-Label "Karte"
    StatusMap,
    /// Status-Label "Zoom"
    StatusZoom,
    /// Status-Label "Position"
    StatusPosition,
    /// Status-Label "Höhenkarte"
    StatusHeightmap,
    /// Status-Wert "Keine" (z. B. bei fehlendem Heightmap)
    StatusHeightmapNone,
    /// Status-Label "Ausgewählte Knoten"
    StatusSelectedNodes,
    /// Abkürzung "z. B." für Status-Beispielwert
    StatusExample,
    /// Status-Label "Werkzeug"
    StatusTool,
    /// Status-Label "FPS"
    StatusFps,

    // === Werkzeug-Namen (Status-Bar) ===
    /// Werkzeugname "Auswahl"
    ToolNameSelect,
    /// Werkzeugname "Verbinden"
    ToolNameConnect,
    /// Werkzeugname "Knoten hinzufügen"
    ToolNameAddNode,
    /// Werkzeugname "Routen-Werkzeug"
    ToolNameRoute,

    // === Sidebar: Sections ===
    /// Abschnittstitel "Werkzeuge"
    SidebarTools,
    /// Abschnittstitel "Grundbefehle"
    SidebarBasics,
    /// Abschnittstitel "Bearbeiten"
    SidebarEdit,
    /// Abschnittstitel "Richtung"
    SidebarDirection,
    /// Abschnittstitel "Strassenart"
    SidebarPriority,
    /// Abschnittstitel "Zoom"
    SidebarZoom,
    /// Abschnittstitel "Hintergrund"
    SidebarBackground,

    // === Zoom ===
    /// Button-Label "Auf komplette Map"
    ZoomFullMap,
    /// Tooltip "Gesamte Map einpassen"
    ZoomFullMapHelp,
    /// Button-Label "Auf Auswahl"
    ZoomToSelection,
    /// Tooltip "Auf selektierte Nodes einpassen"
    ZoomToSelectionHelp,

    // === Hintergrund ===
    /// Button-Tooltip "Hintergrund ausblenden"
    BackgroundHide,
    /// Button-Tooltip "Hintergrund einblenden"
    BackgroundShow,
    /// Button-Tooltip "Ausdehnung halbieren"
    BackgroundScaleDown,
    /// Button-Tooltip "Ausdehnung verdoppeln"
    BackgroundScaleUp,
    /// Button-Tooltip "Originalgrösse"
    BackgroundScaleReset,

    // === Sidebar: Route-Gruppen ===
    /// Gruppenname "Geraden"
    RouteGroupStraight,
    /// Gruppenname "Kurven"
    RouteGroupCurves,
    /// Gruppenname "Tools"
    RouteGroupSection,

    // === Floating-Menu: Tools ===
    /// Tooltip "Auswahl"
    FloatingToolSelect,
    /// Tooltip "Verbinden"
    FloatingToolConnect,
    /// Tooltip "Node hinzufügen"
    FloatingToolAddNode,

    // === Floating-Menu: Grundbefehle ===
    /// Tooltip "Gerade Strecke"
    FloatingBasicStraight,
    /// Tooltip "Bezier Grad 2"
    FloatingBasicQuadratic,
    /// Tooltip "Bezier Grad 3"
    FloatingBasicCubic,
    /// Tooltip "Spline"
    FloatingBasicSpline,
    /// Tooltip "Geglaettete Kurve"
    FloatingBasicSmoothCurve,

    // === Floating-Menu: Bearbeiten ===
    /// Tooltip "Ausweichstrecke"
    FloatingEditBypass,
    /// Tooltip "Parkplatz"
    FloatingEditParking,
    /// Tooltip "Strecke versetzen"
    FloatingEditRouteOffset,

    // === Floating-Menu: Richtung + Strassenart ===
    /// Tooltip "Einbahn vorwaerts"
    FloatingDirectionRegular,
    /// Tooltip "Zweirichtungsverkehr"
    FloatingDirectionDual,
    /// Tooltip "Einbahn rueckwaerts"
    FloatingDirectionReverse,
    /// Tooltip "Hauptstrasse"
    FloatingPriorityMain,
    /// Tooltip "Nebenstrasse"
    FloatingPrioritySub,

    // === Floating-Menu: Zoom ===
    /// Tooltip "Auf komplette Map"
    FloatingZoomFullMap,
    /// Tooltip "Auf Auswahl"
    FloatingZoomSelection,

    // === Kontextmenue ===
    /// Submenu-Titel "Werkzeug"
    CtxToolSubmenu,
    /// Eintrag "Auswahl (T)"
    CtxToolSelect,
    /// Eintrag "Verbinden (T)"
    CtxToolConnect,
    /// Eintrag "Node hinzufuegen (T)"
    CtxToolAddNode,
    /// Submenu-Titel "Zoom"
    CtxZoomSubmenu,
    /// Eintrag "Auf komplette Map (Z)"
    CtxZoomFullMap,
    /// Eintrag "Auf Auswahl (Z)"
    CtxZoomSelection,
    /// Submenu-Titel "Strecke"
    CtxRouteSubmenu,
    /// Eintrag "Geglaettete Kurve"
    CtxRouteSmoothCurve,
    /// Eintrag "Gerade Strecke"
    CtxRouteStraight,
    /// Eintrag "Bezier Grad 2"
    CtxRouteQuadratic,
    /// Eintrag "Bezier Grad 3"
    CtxRouteCubic,
    /// Eintrag "Segment bearbeiten"
    CtxEditSegment,
    /// Eintrag "Als Segment gruppieren"
    CtxGroupAsSegment,
    /// Eintrag "Nodes verbinden"
    CtxConnectNodes,
    /// Eintrag "Strecke erzeugen"
    CtxCreateRoute,
    /// Submenu-Titel "Richtung"
    CtxDirectionSubmenu,
    /// Eintrag "Einbahn vorwaerts"
    CtxDirectionRegular,
    /// Eintrag "Zweirichtungsverkehr"
    CtxDirectionDual,
    /// Eintrag "Einbahn rueckwaerts"
    CtxDirectionReverse,
    /// Eintrag "Invertieren"
    CtxDirectionInvert,
    /// Submenu-Titel "Strassenart"
    CtxPrioritySubmenu,
    /// Eintrag "Hauptstrasse"
    CtxPriorityMain,
    /// Eintrag "Nebenstrasse"
    CtxPrioritySub,
    /// Eintrag "Alle trennen"
    CtxRemoveAllConnections,
    /// Submenu-Titel "Selektion"
    CtxSelectionSubmenu,
    /// Eintrag "Invertieren"
    CtxSelectionInvert,
    /// Eintrag "Alles auswaehlen"
    CtxSelectAll,
    /// Eintrag "Auswahl aufheben"
    CtxClearSelection,
    /// Eintrag "Streckenteilung"
    CtxStreckenteilung,
    /// Eintrag "Loeschen"
    CtxDeleteSelected,
    /// Eintrag "Kopieren"
    CtxCopy,
    /// Eintrag "Einfuegen"
    CtxPaste,

    // === Command Palette ===
    /// Placeholder "Befehl eingeben..."
    PaletteSearchHint,
    /// Nachricht "Keine Treffer"
    PaletteNoResults,
    /// Eintrag "Datei oeffnen"
    PaletteOpenFile,
    /// Eintrag "Speichern"
    PaletteSave,
    /// Eintrag "Rueckgaengig"
    PaletteUndo,
    /// Eintrag "Wiederholen"
    PaletteRedo,
    /// Eintrag "Alles auswaehlen"
    PaletteSelectAll,
    /// Eintrag "Auswahl loeschen"
    PaletteDeleteSelected,
    /// Eintrag "Kopieren"
    PaletteCopy,
    /// Eintrag "Einfuegen"
    PalettePaste,
    /// Eintrag "Kamera zuruecksetzen"
    PaletteResetCamera,
    /// Eintrag "Select-Tool"
    PaletteToolSelect,
    /// Eintrag "Connect-Tool"
    PaletteToolConnect,
    /// Eintrag "Add-Node-Tool"
    PaletteToolAddNode,
    /// Praefix "Route-Tool:"
    PaletteRouteToolPrefix,

    // === Sidebar: LongPress-Tooltips ===
    /// LongPress-Tooltip fuer Select-Tool
    LpToolSelect,
    /// LongPress-Tooltip fuer Connect-Tool
    LpToolConnect,
    /// LongPress-Tooltip fuer AddNode-Tool
    LpToolAddNode,
    /// LongPress-Tooltip fuer Gerade Strecke
    LpStraight,
    /// LongPress-Tooltip fuer Bezier quadratisch
    LpCurveQuad,
    /// LongPress-Tooltip fuer Bezier kubisch
    LpCurveCubic,
    /// LongPress-Tooltip fuer Spline
    LpSpline,
    /// LongPress-Tooltip fuer Geglaettete Kurve
    LpSmoothCurve,
    /// LongPress-Tooltip fuer Ausweichstrecke
    LpBypass,
    /// LongPress-Tooltip fuer Parkplatz
    LpParking,
    /// LongPress-Tooltip fuer Strecke versetzen
    LpRouteOffset,
    /// LongPress-Tooltip fuer Einbahn vorwaerts
    LpDirectionRegular,
    /// LongPress-Tooltip fuer Zweirichtung
    LpDirectionDual,
    /// LongPress-Tooltip fuer Einbahn rueckwaerts
    LpDirectionReverse,
    /// LongPress-Tooltip fuer Hauptstrasse
    LpPriorityMain,
    /// LongPress-Tooltip fuer Nebenstrasse
    LpPrioritySub,
}
