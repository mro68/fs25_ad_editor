macro_rules! route_tool_command_variants {
    () => {
        /// Route-Tool: Viewport-Klick verarbeiten
        RouteToolClick { world_pos: glam::Vec2, ctrl: bool },
        /// Route-Tool: Ergebnis anwenden
        RouteToolExecute,
        /// Route-Tool: Abbrechen
        RouteToolCancel,
        /// Route-Tool per stabiler Tool-ID aktivieren.
        SelectRouteTool { tool_id: RouteToolId },
        /// Route-Tool mit vordefinierten Start/End-Nodes aktivieren und Klicks simulieren
        RouteToolWithAnchors {
            tool_id: RouteToolId,
            start_node_id: u64,
            end_node_id: u64,
        },
        /// Route-Tool: Strecke neu berechnen (Config geaendert)
        RouteToolRecreate,
        /// Route-Tool: Semantische Panel-Aktion anwenden.
        RouteToolPanelAction { action: RouteToolPanelAction },
        /// Route-Tool: Node-Anzahl erhoehen
        IncreaseRouteToolNodeCount,
        /// Route-Tool: Node-Anzahl verringern
        DecreaseRouteToolNodeCount,
        /// Route-Tool: Minimalabstand um 0.25m erhoehen
        IncreaseRouteToolSegmentLength,
        /// Route-Tool: Minimalabstand um 0.25m verringern
        DecreaseRouteToolSegmentLength,
        /// Route-Tool: Tangenten-Auswahl anwenden und ggf. Recreate triggern
        RouteToolApplyTangent {
            start: TangentSource,
            end: TangentSource,
        },
        /// Route-Tool: Lasso-Polygon an das aktive Route-Tool weiterleiten
        RouteToolLassoCompleted { polygon: Vec<glam::Vec2> },
        /// Route-Tool: Drag auf Steuerpunkt/Anker starten
        RouteToolDragStart { world_pos: glam::Vec2 },
        /// Route-Tool: Drag-Position aktualisieren
        RouteToolDragUpdate { world_pos: glam::Vec2 },
        /// Route-Tool: Drag beenden
        RouteToolDragEnd,
        /// Route-Tool: Scroll-Rotation anwenden
        RouteToolRotate { delta: f32 },
    };
}

pub(super) use route_tool_command_variants;

macro_rules! route_tool_command_feature_arms {
    () => {
        Self::RouteToolClick { .. }
        | Self::RouteToolExecute
        | Self::RouteToolCancel
        | Self::SelectRouteTool { .. }
        | Self::RouteToolWithAnchors { .. }
        | Self::RouteToolRecreate
        | Self::RouteToolPanelAction { .. }
        | Self::IncreaseRouteToolNodeCount
        | Self::DecreaseRouteToolNodeCount
        | Self::IncreaseRouteToolSegmentLength
        | Self::DecreaseRouteToolSegmentLength
        | Self::RouteToolApplyTangent { .. }
        | Self::RouteToolLassoCompleted { .. }
        | Self::RouteToolDragStart { .. }
        | Self::RouteToolDragUpdate { .. }
        | Self::RouteToolDragEnd
        | Self::RouteToolRotate { .. } => AppEventFeature::RouteTool,
    };
}

pub(super) use route_tool_command_feature_arms;