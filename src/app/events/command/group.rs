macro_rules! group_command_variants {
    () => {
        /// Segment nachtraeglich bearbeiten
        EditGroup { record_id: u64 },
        /// Segment-Lock umschalten (gesperrt ↔ entsperrt)
        ToggleGroupLock { segment_id: u64 },
        /// Segment aufloesen (Segment-Record entfernen, Nodes beibehalten)
        DissolveGroup { segment_id: u64 },
        /// Dialog zum Bestaetigen des Aufloesens oeffnen
        OpenDissolveConfirmDialog { segment_id: u64 },
        /// Selektierte zusammenhaengende Nodes als neues Segment in der Registry speichern
        GroupSelectionAsGroup,
        /// Selektierte Nodes aus ihren zugehoerigen Gruppen entfernen
        RemoveSelectedNodesFromGroups,
        /// Einfahrt/Ausfahrt-Nodes einer Gruppe setzen
        SetGroupBoundaryNodes {
            record_id: u64,
            entry_node_id: Option<u64>,
            exit_node_id: Option<u64>,
        },
        /// Gruppen-Edit-Modus nicht-destruktiv starten
        GroupEditStart { record_id: u64 },
        /// Gruppen-Edit uebernehmen (Aenderungen persistieren)
        GroupEditApply,
        /// Gruppen-Edit abbrechen (Undo zum Snapshot)
        GroupEditCancel,
        /// Atomar: Gruppen-Edit aufraumen → Undo → Tool-Edit starten
        BeginToolEditFromGroup { record_id: u64 },
        /// Segment-Einstellungs-Popup oeffnen oder aktualisieren
        OpenGroupSettingsPopup { world_pos: glam::Vec2 },
    };
}

pub(super) use group_command_variants;

macro_rules! group_command_feature_arms {
    () => {
        Self::EditGroup { .. }
        | Self::ToggleGroupLock { .. }
        | Self::DissolveGroup { .. }
        | Self::OpenDissolveConfirmDialog { .. }
        | Self::GroupSelectionAsGroup
        | Self::RemoveSelectedNodesFromGroups
        | Self::SetGroupBoundaryNodes { .. }
        | Self::GroupEditStart { .. }
        | Self::GroupEditApply
        | Self::GroupEditCancel
        | Self::BeginToolEditFromGroup { .. }
        | Self::OpenGroupSettingsPopup { .. } => AppEventFeature::Group,
    };
}

pub(super) use group_command_feature_arms;