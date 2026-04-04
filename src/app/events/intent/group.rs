macro_rules! group_intent_variants {
    () => {
        /// Segment nachtraeglich bearbeiten (Nodes loeschen + Tool laden)
        EditGroupRequested { record_id: u64 },
        /// Gruppen-Bearbeitung nicht-destruktiv starten (Select-Tool-Modus)
        GroupEditStartRequested { record_id: u64 },
        /// Gruppen-Bearbeitung abschliessen (Aenderungen uebernehmen)
        GroupEditApplyRequested,
        /// Gruppen-Bearbeitung abbrechen (Undo zum Snapshot vor Edit-Start)
        GroupEditCancelRequested,
        /// Aus Gruppen-Edit heraus das Tool-Edit starten (destruktiv/regenerativ)
        GroupEditToolRequested { record_id: u64 },
        /// Segment-Lock umschalten (gesperrt ↔ entsperrt)
        ToggleGroupLockRequested { segment_id: u64 },
        /// Segment aufloesen (Segment-Record entfernen, Nodes beibehalten)
        DissolveGroupRequested { segment_id: u64 },
        /// Bestaetigung: Gruppe aufloesen (nach Dialog-Bestaetigung)
        DissolveGroupConfirmed { segment_id: u64 },
        /// Selektierte zusammenhaengende Nodes als neues Segment in der Registry speichern
        GroupSelectionAsGroupRequested,
        /// Selektierte Nodes aus ihrer Gruppe entfernen (Nodes bleiben in RoadMap erhalten)
        RemoveSelectedNodesFromGroupRequested,
        /// Einfahrt/Ausfahrt-Nodes einer Gruppe setzen
        SetGroupBoundaryNodes {
            record_id: u64,
            entry_node_id: Option<u64>,
            exit_node_id: Option<u64>,
        },
    };
}

pub(super) use group_intent_variants;

macro_rules! group_intent_feature_arms {
    () => {
        Self::EditGroupRequested { .. }
        | Self::GroupEditStartRequested { .. }
        | Self::GroupEditApplyRequested
        | Self::GroupEditCancelRequested
        | Self::GroupEditToolRequested { .. }
        | Self::GroupSelectionAsGroupRequested
        | Self::RemoveSelectedNodesFromGroupRequested
        | Self::SetGroupBoundaryNodes { .. }
        | Self::ToggleGroupLockRequested { .. }
        | Self::DissolveGroupRequested { .. }
        | Self::DissolveGroupConfirmed { .. } => AppEventFeature::Group,
    };
}

pub(super) use group_intent_feature_arms;