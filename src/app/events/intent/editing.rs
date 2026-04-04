macro_rules! editing_intent_variants {
    () => {
        /// Editor-Werkzeug wechseln
        SetEditorToolRequested { tool: EditorTool },
        /// Neuen Node an Weltposition hinzufuegen
        AddNodeRequested { world_pos: glam::Vec2 },
        /// Selektierte Nodes loeschen
        DeleteSelectedRequested,
        /// Connect-Tool: Node angeklickt (Source oder Target)
        ConnectToolNodeClicked { world_pos: glam::Vec2 },
        /// Verbindung zwischen zwei Nodes erstellen (via Shortcut/Panel)
        AddConnectionRequested {
            from_id: u64,
            to_id: u64,
            direction: ConnectionDirection,
            priority: ConnectionPriority,
        },
        /// Alle Verbindungen zwischen zwei Nodes entfernen
        RemoveConnectionBetweenRequested { node_a: u64, node_b: u64 },
        /// Richtung einer Verbindung aendern
        SetConnectionDirectionRequested {
            start_id: u64,
            end_id: u64,
            direction: ConnectionDirection,
        },
        /// Prioritaet einer Verbindung aendern
        SetConnectionPriorityRequested {
            start_id: u64,
            end_id: u64,
            priority: ConnectionPriority,
        },
        /// Node-Flag aendern (Regular, SubPrio, etc.)
        NodeFlagChangeRequested { node_id: u64, flag: NodeFlag },
        /// Standard-Richtung fuer neue Verbindungen aendern
        SetDefaultDirectionRequested { direction: ConnectionDirection },
        /// Standard-Strassenart fuer neue Verbindungen aendern
        SetDefaultPriorityRequested { priority: ConnectionPriority },
        /// Richtung aller Verbindungen zwischen selektierten Nodes aendern
        SetAllConnectionsDirectionBetweenSelectedRequested { direction: ConnectionDirection },
        /// Alle Verbindungen zwischen selektierten Nodes trennen
        RemoveAllConnectionsBetweenSelectedRequested,
        /// Richtung aller Verbindungen zwischen selektierten Nodes invertieren (start↔end tauschen)
        InvertAllConnectionsBetweenSelectedRequested,
        /// Prioritaet aller Verbindungen zwischen selektierten Nodes aendern
        SetAllConnectionsPriorityBetweenSelectedRequested { priority: ConnectionPriority },
        /// Zwei selektierte Nodes verbinden (mit Standard-Richtung/Prioritaet)
        ConnectSelectedNodesRequested,
        /// Map-Marker fuer einen Node erstellen
        CreateMarkerRequested { node_id: u64 },
        /// Map-Marker fuer einen Node entfernen
        RemoveMarkerRequested { node_id: u64 },
        /// Map-Marker bearbeiten (Dialog oeffnen)
        EditMarkerRequested { node_id: u64 },
        /// Marker-Dialog bestaetigt (erstellen oder aktualisieren)
        MarkerDialogConfirmed {
            node_id: u64,
            name: String,
            group: String,
            is_new: bool,
        },
        /// Marker-Dialog abgebrochen
        MarkerDialogCancelled,
        /// Selektierte Nodes-Kette als gleichmaessig verteilte Wegpunkte neu berechnen (Distanzen)
        ResamplePathRequested,
        /// Streckenteilung-Panel aktivieren (z.B. per Kontextmenue)
        StreckenteilungAktivieren,
        /// Selektion in die Zwischenablage kopieren
        CopySelectionRequested,
        /// Einfuegen-Vorschau starten (Clipboard → Vorschau auf Karte)
        PasteStartRequested,
        /// Einfuegen-Vorschau: Mauszeiger hat sich bewegt → Vorschau aktualisieren
        PastePreviewMoved { world_pos: glam::Vec2 },
        /// Einfuegen an aktueller Vorschauposition bestaetigen
        PasteConfirmRequested,
        /// Einfuegen-Vorschau abbrechen (Escape)
        PasteCancelled,
        /// Alle-Felder-nachzeichnen-Einstellungsdialog oeffnen
        OpenTraceAllFieldsDialogRequested,
        /// Alle-Felder-nachzeichnen bestaetigt (nach Dialog-Eingabe)
        TraceAllFieldsConfirmed {
            spacing: f32,
            offset: f32,
            tolerance: f32,
            corner_angle: Option<f32>,
            corner_rounding_radius: Option<f32>,
            corner_rounding_max_angle_deg: Option<f32>,
        },
        /// Alle-Felder-nachzeichnen-Dialog abgebrochen
        TraceAllFieldsCancelled,
        /// Curseplay-Import-Dialog anfordern
        CurseplayImportRequested,
        /// Curseplay-Export-Dialog anfordern
        CurseplayExportRequested,
        /// Curseplay-Importdatei wurde im Dialog ausgewaehlt
        CurseplayFileSelected { path: String },
        /// Curseplay-Exportpfad wurde im Dialog ausgewaehlt
        CurseplayExportPathSelected { path: String },
    };
}

pub(super) use editing_intent_variants;

macro_rules! editing_intent_feature_arms {
    () => {
        Self::SetEditorToolRequested { .. }
        | Self::AddNodeRequested { .. }
        | Self::DeleteSelectedRequested
        | Self::ConnectToolNodeClicked { .. }
        | Self::AddConnectionRequested { .. }
        | Self::RemoveConnectionBetweenRequested { .. }
        | Self::SetConnectionDirectionRequested { .. }
        | Self::SetConnectionPriorityRequested { .. }
        | Self::NodeFlagChangeRequested { .. }
        | Self::SetDefaultDirectionRequested { .. }
        | Self::SetDefaultPriorityRequested { .. }
        | Self::SetAllConnectionsDirectionBetweenSelectedRequested { .. }
        | Self::RemoveAllConnectionsBetweenSelectedRequested
        | Self::InvertAllConnectionsBetweenSelectedRequested
        | Self::SetAllConnectionsPriorityBetweenSelectedRequested { .. }
        | Self::ConnectSelectedNodesRequested
        | Self::CreateMarkerRequested { .. }
        | Self::RemoveMarkerRequested { .. }
        | Self::EditMarkerRequested { .. }
        | Self::MarkerDialogConfirmed { .. }
        | Self::MarkerDialogCancelled
        | Self::ResamplePathRequested
        | Self::StreckenteilungAktivieren
        | Self::CopySelectionRequested
        | Self::PasteStartRequested
        | Self::PastePreviewMoved { .. }
        | Self::PasteConfirmRequested
        | Self::PasteCancelled
        | Self::OpenTraceAllFieldsDialogRequested
        | Self::TraceAllFieldsConfirmed { .. }
        | Self::TraceAllFieldsCancelled
        | Self::CurseplayImportRequested
        | Self::CurseplayExportRequested
        | Self::CurseplayFileSelected { .. }
        | Self::CurseplayExportPathSelected { .. } => AppEventFeature::Editing,
    };
}

pub(super) use editing_intent_feature_arms;