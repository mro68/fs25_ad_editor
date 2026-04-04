macro_rules! selection_intent_variants {
    () => {
        /// Node per Klick selektieren (Nearest-Node-Pick)
        NodePickRequested {
            world_pos: glam::Vec2,
            additive: bool,
            extend_path: bool,
        },
        /// Segment zwischen Kreuzungen per Doppelklick selektieren
        NodeSegmentBetweenIntersectionsRequested {
            world_pos: glam::Vec2,
            additive: bool,
        },
        /// Nodes innerhalb eines Rechtecks selektieren (Shift + Drag)
        SelectNodesInRectRequested {
            min: glam::Vec2,
            max: glam::Vec2,
            additive: bool,
        },
        /// Nodes innerhalb eines Lasso-Polygons selektieren (Alt + Drag)
        SelectNodesInLassoRequested {
            polygon: Vec<glam::Vec2>,
            additive: bool,
        },
        /// Move-Lifecycle Start: Drag-Verschieben selektierter Nodes beginnen
        BeginMoveSelectedNodesRequested,
        /// Move-Lifecycle Update: Selektierte Nodes um Delta verschieben
        MoveSelectedNodesRequested { delta_world: glam::Vec2 },
        /// Move-Lifecycle Ende: Drag-Verschieben abgeschlossen
        EndMoveSelectedNodesRequested,
        /// Rotation-Lifecycle Start: Undo-Snapshot aufnehmen
        BeginRotateSelectedNodesRequested,
        /// Rotation-Lifecycle Update: Selektierte Nodes um Delta-Winkel (Radiant) rotieren
        RotateSelectedNodesRequested { delta_angle: f32 },
        /// Rotation-Lifecycle Ende: Spatial-Index rebuild ausloesen
        EndRotateSelectedNodesRequested,
        /// Selektion aufheben
        ClearSelectionRequested,
        /// Alle Nodes selektieren
        SelectAllRequested,
        /// Auswahl invertieren (selektierte abwaehlen, nicht-selektierte waehlen)
        InvertSelectionRequested,
    };
}

pub(super) use selection_intent_variants;

macro_rules! selection_intent_feature_arms {
    () => {
        Self::NodePickRequested { .. }
        | Self::NodeSegmentBetweenIntersectionsRequested { .. }
        | Self::SelectNodesInRectRequested { .. }
        | Self::SelectNodesInLassoRequested { .. }
        | Self::BeginMoveSelectedNodesRequested
        | Self::MoveSelectedNodesRequested { .. }
        | Self::EndMoveSelectedNodesRequested
        | Self::BeginRotateSelectedNodesRequested
        | Self::RotateSelectedNodesRequested { .. }
        | Self::EndRotateSelectedNodesRequested
        | Self::ClearSelectionRequested
        | Self::SelectAllRequested
        | Self::InvertSelectionRequested => AppEventFeature::Selection,
    };
}

pub(super) use selection_intent_feature_arms;