macro_rules! selection_command_variants {
    () => {
        /// Naechsten Node zur Position selektieren
        SelectNearestNode {
            world_pos: glam::Vec2,
            max_distance: f32,
            additive: bool,
            extend_path: bool,
        },
        /// Segment zwischen Kreuzungen selektieren
        SelectSegmentBetweenNearestIntersections {
            world_pos: glam::Vec2,
            max_distance: f32,
            additive: bool,
            stop_at_junction: bool,
            max_angle_deg: f32,
        },
        /// Alle Nodes einer Gruppe selektieren (identifiziert ueber Naehe zu world_pos)
        SelectGroupByNearestNode {
            world_pos: glam::Vec2,
            max_distance: f32,
            additive: bool,
        },
        /// Nodes innerhalb eines Rechtecks selektieren
        SelectNodesInRect {
            min: glam::Vec2,
            max: glam::Vec2,
            additive: bool,
        },
        /// Nodes innerhalb eines Lasso-Polygons selektieren
        SelectNodesInLasso {
            polygon: Vec<glam::Vec2>,
            additive: bool,
        },
        /// Selektierte Nodes um Delta verschieben
        MoveSelectedNodes { delta_world: glam::Vec2 },
        /// Move-Lifecycle: Verschieben starten (Undo-Snapshot)
        BeginMoveSelectedNodes,
        /// Move-Lifecycle: Verschieben beenden
        EndMoveSelectedNodes,
        /// Rotation-Lifecycle: Starten (Undo-Snapshot aufnehmen)
        BeginRotateSelectedNodes,
        /// Rotation-Lifecycle: Selektierte Nodes um Delta-Winkel (Radiant) rotieren
        RotateSelectedNodes { delta_angle: f32 },
        /// Rotation-Lifecycle: Beenden (Spatial-Index rebuild anstoßen)
        EndRotateSelectedNodes,
        /// Selektion aufheben
        ClearSelection,
        /// Alle Nodes selektieren
        SelectAllNodes,
        /// Auswahl invertieren
        InvertSelection,
    };
}

pub(super) use selection_command_variants;

macro_rules! selection_command_feature_arms {
    () => {
        Self::SelectNearestNode { .. }
        | Self::SelectSegmentBetweenNearestIntersections { .. }
        | Self::SelectGroupByNearestNode { .. }
        | Self::SelectNodesInRect { .. }
        | Self::SelectNodesInLasso { .. }
        | Self::MoveSelectedNodes { .. }
        | Self::BeginMoveSelectedNodes
        | Self::EndMoveSelectedNodes
        | Self::BeginRotateSelectedNodes
        | Self::RotateSelectedNodes { .. }
        | Self::EndRotateSelectedNodes
        | Self::ClearSelection
        | Self::SelectAllNodes
        | Self::InvertSelection => AppEventFeature::Selection,
    };
}

pub(super) use selection_command_feature_arms;