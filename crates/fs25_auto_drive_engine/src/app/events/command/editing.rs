macro_rules! editing_command_variants {
    () => {
        /// Editor-Werkzeug wechseln
        SetEditorTool { tool: EditorTool },
        /// Neuen Node an Weltposition hinzufuegen
        AddNodeAtPosition { world_pos: glam::Vec2 },
        /// Selektierte Nodes loeschen
        DeleteSelectedNodes,
        /// Connect-Tool: Node anwaehlen (Source oder Target)
        ConnectToolPickNode {
            world_pos: glam::Vec2,
            max_distance: f32,
        },
        /// Verbindung zwischen zwei Nodes erstellen
        AddConnection {
            from_id: u64,
            to_id: u64,
            direction: ConnectionDirection,
            priority: ConnectionPriority,
        },
        /// Alle Verbindungen zwischen zwei Nodes entfernen
        RemoveConnectionBetween { node_a: u64, node_b: u64 },
        /// Richtung einer Verbindung aendern
        SetConnectionDirection {
            start_id: u64,
            end_id: u64,
            direction: ConnectionDirection,
        },
        /// Prioritaet einer Verbindung aendern
        SetConnectionPriority {
            start_id: u64,
            end_id: u64,
            priority: ConnectionPriority,
        },
        /// Setzt das Flag eines Nodes
        SetNodeFlag { node_id: u64, flag: NodeFlag },
        /// Standard-Richtung fuer neue Verbindungen setzen
        SetDefaultDirection { direction: ConnectionDirection },
        /// Standard-Prioritaet fuer neue Verbindungen setzen
        SetDefaultPriority { priority: ConnectionPriority },
        /// Bulk: Richtung aller Verbindungen zwischen Selektion aendern
        SetAllConnectionsDirectionBetweenSelected { direction: ConnectionDirection },
        /// Bulk: Alle Verbindungen zwischen Selektion entfernen
        RemoveAllConnectionsBetweenSelected,
        /// Bulk: Richtung aller Verbindungen zwischen Selektion invertieren
        InvertAllConnectionsBetweenSelected,
        /// Bulk: Prioritaet aller Verbindungen zwischen Selektion aendern
        SetAllConnectionsPriorityBetweenSelected { priority: ConnectionPriority },
        /// Zwei selektierte Nodes mit Standard-Einstellungen verbinden
        ConnectSelectedNodes,
        /// Map-Marker erstellen
        CreateMarker {
            node_id: u64,
            name: String,
            group: String,
        },
        /// Map-Marker entfernen
        RemoveMarker { node_id: u64 },
        /// Marker-Dialog oeffnen (neu oder bearbeiten)
        OpenMarkerDialog { node_id: u64, is_new: bool },
        /// Marker aktualisieren
        UpdateMarker {
            node_id: u64,
            name: String,
            group: String,
        },
        /// Selektierte Nodes-Kette als gleichmaessig verteilte Wegpunkte neu berechnen (Distanzen)
        ResamplePath,
        /// Streckenteilung-Panel aktivieren
        StreckenteilungAktivieren,
        /// Selektion in die Zwischenablage kopieren
        CopySelection,
        /// Einfuegen-Vorschau starten
        StartPastePreview,
        /// Einfuegen-Vorschau: Position aktualisieren
        UpdatePastePreview { world_pos: glam::Vec2 },
        /// Einfuegen an aktueller Vorschauposition bestaetigen
        ConfirmPaste,
        /// Einfuegen-Vorschau abbrechen
        CancelPastePreview,
        /// Alle Farmland-Polygone als Wegpunkt-Ring nachzeichnen (Batch-Operation)
        TraceAllFields {
            spacing: f32,
            offset: f32,
            tolerance: f32,
            corner_angle: Option<f32>,
            corner_rounding_radius: Option<f32>,
            corner_rounding_max_angle_deg: Option<f32>,
        },
        /// Curseplay-Datei importieren (Nodes + Ring-Verbindungen anlegen)
        ImportCurseplay { path: String },
        /// Selektierte Strecke als Curseplay-XML exportieren
        ExportCurseplay { path: String },
    };
}

pub(super) use editing_command_variants;

macro_rules! editing_command_feature_arms {
    () => {
        Self::SetEditorTool { .. }
        | Self::AddNodeAtPosition { .. }
        | Self::DeleteSelectedNodes
        | Self::ConnectToolPickNode { .. }
        | Self::AddConnection { .. }
        | Self::RemoveConnectionBetween { .. }
        | Self::SetConnectionDirection { .. }
        | Self::SetConnectionPriority { .. }
        | Self::SetNodeFlag { .. }
        | Self::SetDefaultDirection { .. }
        | Self::SetDefaultPriority { .. }
        | Self::SetAllConnectionsDirectionBetweenSelected { .. }
        | Self::RemoveAllConnectionsBetweenSelected
        | Self::InvertAllConnectionsBetweenSelected
        | Self::SetAllConnectionsPriorityBetweenSelected { .. }
        | Self::ConnectSelectedNodes
        | Self::CreateMarker { .. }
        | Self::RemoveMarker { .. }
        | Self::OpenMarkerDialog { .. }
        | Self::UpdateMarker { .. }
        | Self::ResamplePath
        | Self::StreckenteilungAktivieren
        | Self::CopySelection
        | Self::StartPastePreview
        | Self::UpdatePastePreview { .. }
        | Self::ConfirmPaste
        | Self::CancelPastePreview
        | Self::TraceAllFields { .. }
        | Self::ImportCurseplay { .. }
        | Self::ExportCurseplay { .. } => AppEventFeature::Editing,
    };
}

pub(super) use editing_command_feature_arms;