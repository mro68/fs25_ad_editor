macro_rules! route_tool_intent_variants {
    () => {
        /// Route-Tool: Viewport-Klick
        RouteToolClicked { world_pos: glam::Vec2, ctrl: bool },
        /// Route-Tool: Ausfuehrung bestaetigt (Enter)
        RouteToolExecuteRequested,
        /// Route-Tool: Abbrechen (Escape)
        RouteToolCancelled,
        /// Route-Tool auswaehlen (ueber stabile Tool-ID im Katalog)
        SelectRouteToolRequested { tool_id: RouteToolId },
        /// Route-Tool mit vordefinierten Start/End-Nodes aktivieren (Kontextmenue bei 2 selektierten Nodes)
        RouteToolWithAnchorsRequested {
            tool_id: RouteToolId,
            start_node_id: u64,
            end_node_id: u64,
        },
        /// Route-Tool: Konfiguration geaendert (Distanz/Anzahl) → Strecke neu berechnen
        RouteToolConfigChanged,
        /// Route-Tool: Semantische Panel-Aktion aus dem schwebenden Panel.
        RouteToolPanelActionRequested { action: RouteToolPanelAction },
        /// Route-Tool: Tangenten-Auswahl aus dem Kontextmenue aendern
        RouteToolTangentSelected {
            start: TangentSource,
            end: TangentSource,
        },
        /// Route-Tool: Lasso-Polygon abgeschlossen (Alt+Drag bei Tools die `needs_lasso_input()` setzen)
        RouteToolLassoCompleted { polygon: Vec<glam::Vec2> },
        /// Route-Tool: Drag auf Steuerpunkt/Anker gestartet
        RouteToolDragStarted { world_pos: glam::Vec2 },
        /// Route-Tool: Drag-Position aktualisiert
        RouteToolDragUpdated { world_pos: glam::Vec2 },
        /// Route-Tool: Drag beendet (Punkt loslassen)
        RouteToolDragEnded,
        /// Route-Tool: Alt+Scroll-Rotation
        RouteToolScrollRotated { delta: f32 },
        /// Route-Tool: Strecke neu berechnen mit aktuellem Config (nach Parameter-Aenderung)
        RouteToolRecreateRequested,
        /// Route-Tool: Node-Anzahl erhoehen (Pfeiltaste oben)
        IncreaseRouteToolNodeCount,
        /// Route-Tool: Node-Anzahl verringern (Pfeiltaste unten)
        DecreaseRouteToolNodeCount,
        /// Route-Tool: Minimalabstand um 0.25m erhoehen (Pfeiltaste rechts)
        IncreaseRouteToolSegmentLength,
        /// Route-Tool: Minimalabstand um 0.25m verringern (Pfeiltaste links)
        DecreaseRouteToolSegmentLength,
    };
}

pub(super) use route_tool_intent_variants;

macro_rules! route_tool_intent_feature_arms {
    () => {
        Self::RouteToolClicked { .. }
        | Self::RouteToolExecuteRequested
        | Self::RouteToolCancelled
        | Self::SelectRouteToolRequested { .. }
        | Self::RouteToolWithAnchorsRequested { .. }
        | Self::RouteToolConfigChanged
        | Self::RouteToolPanelActionRequested { .. }
        | Self::RouteToolTangentSelected { .. }
        | Self::RouteToolLassoCompleted { .. }
        | Self::RouteToolDragStarted { .. }
        | Self::RouteToolDragUpdated { .. }
        | Self::RouteToolDragEnded
        | Self::RouteToolScrollRotated { .. }
        | Self::RouteToolRecreateRequested
        | Self::IncreaseRouteToolNodeCount
        | Self::DecreaseRouteToolNodeCount
        | Self::IncreaseRouteToolSegmentLength
        | Self::DecreaseRouteToolSegmentLength => AppEventFeature::RouteTool,
    };
}

pub(super) use route_tool_intent_feature_arms;