//! ToolManager und Capability-Discovery fuer Route-Tools.

use crate::app::tool_contract::RouteToolId;

use super::{
    route_tool_catalog, route_tool_descriptor, route_tool_slot, OrderedNodeChain, RouteTool,
    RouteToolChainInput, RouteToolDescriptor, RouteToolDrag, RouteToolGroupEdit,
    RouteToolLassoInput, RouteToolRecreate, RouteToolRotate, RouteToolSegmentAdjustments,
    RouteToolTangent, ToolHostContext,
};

/// Verwaltet registrierte Route-Tools und den aktiven Tool-Slot.
pub struct ToolManager {
    tools: Vec<RegisteredTool>,
    active_index: Option<usize>,
}

struct RegisteredTool {
    id: RouteToolId,
    tool: Box<dyn RouteTool>,
}

impl Default for ToolManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolManager {
    /// Erstellt einen neuen ToolManager mit dem kanonischen Route-Tool-Katalog.
    pub fn new() -> Self {
        let mut manager = Self {
            tools: Vec::new(),
            active_index: None,
        };
        for descriptor in route_tool_catalog() {
            manager.register(descriptor.id, (descriptor.factory)());
        }
        manager
    }

    /// Registriert ein neues Route-Tool unter stabiler Tool-ID.
    pub fn register(&mut self, tool_id: RouteToolId, tool: Box<dyn RouteTool>) {
        self.tools.push(RegisteredTool { id: tool_id, tool });
    }

    /// Gibt die Anzahl registrierter Tools zurueck.
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Gibt Name und ID aller registrierten Tools zurueck.
    pub fn tool_names(&self) -> Vec<(RouteToolId, &str)> {
        self.tools
            .iter()
            .map(|entry| {
                let descriptor = route_tool_descriptor(entry.id);
                (entry.id, descriptor.name)
            })
            .collect()
    }

    /// Gibt ID, Namen und Legacy-Icon aller registrierten Tools zurueck.
    pub fn tool_entries(&self) -> Vec<(RouteToolId, &str, &str)> {
        self.tools
            .iter()
            .map(|entry| {
                let descriptor = route_tool_descriptor(entry.id);
                (entry.id, descriptor.name, descriptor.legacy_icon)
            })
            .collect()
    }

    fn set_active_slot(&mut self, index: usize) {
        if index < self.tools.len() {
            if let Some(old) = self.active_index {
                if old != index {
                    self.tools[old].tool.reset();
                }
            }
            self.active_index = Some(index);
        }
    }

    /// Setzt das aktive Tool ueber seine stabile Route-Tool-ID.
    pub fn set_active_by_id(&mut self, tool_id: RouteToolId) {
        if let Some(slot) = route_tool_slot(tool_id) {
            self.set_active_slot(slot);
        }
    }

    /// Gibt die stabile Tool-ID des aktiven Tools zurueck.
    pub fn active_id(&self) -> Option<RouteToolId> {
        self.active_index.map(|index| self.tools[index].id)
    }

    /// Gibt den Descriptor des aktiven Tools zurueck.
    pub fn active_descriptor(&self) -> Option<&'static RouteToolDescriptor> {
        self.active_id().map(route_tool_descriptor)
    }

    /// Gibt eine Referenz auf das aktive Route-Tool zurueck.
    pub fn active_tool(&self) -> Option<&dyn RouteTool> {
        self.active_index
            .map(|index| self.tools[index].tool.as_ref())
    }

    /// Gibt eine mutable Referenz auf das aktive Route-Tool zurueck.
    pub fn active_tool_mut(&mut self) -> Option<&mut dyn RouteTool> {
        let index = self.active_index?;
        Some(self.tools[index].tool.as_mut())
    }

    /// Synchronisiert den Host-Kontext in das aktive Tool.
    pub fn sync_active_host(&mut self, context: &ToolHostContext) {
        if let Some(tool) = self.active_tool_mut() {
            tool.sync_host(context);
        }
    }

    /// Liefert die Recreate-Capability des aktiven Tools, falls vorhanden.
    pub fn active_recreate(&self) -> Option<&dyn RouteToolRecreate> {
        self.active_tool().and_then(|tool| tool.as_recreate())
    }

    /// Liefert die mutable Recreate-Capability des aktiven Tools, falls vorhanden.
    pub fn active_recreate_mut(&mut self) -> Option<&mut dyn RouteToolRecreate> {
        self.active_tool_mut()
            .and_then(|tool| tool.as_recreate_mut())
    }

    /// Liefert die Drag-Capability des aktiven Tools, falls vorhanden.
    pub fn active_drag(&self) -> Option<&dyn RouteToolDrag> {
        self.active_tool().and_then(|tool| tool.as_drag())
    }

    /// Liefert die mutable Drag-Capability des aktiven Tools, falls vorhanden.
    pub fn active_drag_mut(&mut self) -> Option<&mut dyn RouteToolDrag> {
        self.active_tool_mut().and_then(|tool| tool.as_drag_mut())
    }

    /// Liefert die Tangent-Capability des aktiven Tools, falls vorhanden.
    pub fn active_tangent(&self) -> Option<&dyn RouteToolTangent> {
        self.active_tool().and_then(|tool| tool.as_tangent())
    }

    /// Liefert die mutable Tangent-Capability des aktiven Tools, falls vorhanden.
    pub fn active_tangent_mut(&mut self) -> Option<&mut dyn RouteToolTangent> {
        self.active_tool_mut()
            .and_then(|tool| tool.as_tangent_mut())
    }

    /// Liefert die Rotations-Capability des aktiven Tools, falls vorhanden.
    pub fn active_rotate(&self) -> Option<&dyn RouteToolRotate> {
        self.active_tool().and_then(|tool| tool.as_rotate())
    }

    /// Liefert die mutable Rotations-Capability des aktiven Tools, falls vorhanden.
    pub fn active_rotate_mut(&mut self) -> Option<&mut dyn RouteToolRotate> {
        self.active_tool_mut().and_then(|tool| tool.as_rotate_mut())
    }

    /// Liefert die Segment-Adjustments-Capability des aktiven Tools, falls vorhanden.
    pub fn active_segment_adjustments(&self) -> Option<&dyn RouteToolSegmentAdjustments> {
        self.active_tool()
            .and_then(|tool| tool.as_segment_adjustments())
    }

    /// Liefert die mutable Segment-Adjustments-Capability des aktiven Tools, falls vorhanden.
    pub fn active_segment_adjustments_mut(
        &mut self,
    ) -> Option<&mut dyn RouteToolSegmentAdjustments> {
        self.active_tool_mut()
            .and_then(|tool| tool.as_segment_adjustments_mut())
    }

    /// Liefert die Chain-Input-Capability des aktiven Tools, falls vorhanden.
    pub fn active_chain_input(&self) -> Option<&dyn RouteToolChainInput> {
        self.active_tool().and_then(|tool| tool.as_chain_input())
    }

    /// Liefert die mutable Chain-Input-Capability des aktiven Tools, falls vorhanden.
    pub fn active_chain_input_mut(&mut self) -> Option<&mut dyn RouteToolChainInput> {
        self.active_tool_mut()
            .and_then(|tool| tool.as_chain_input_mut())
    }

    /// Liefert die Lasso-Capability des aktiven Tools, falls vorhanden.
    pub fn active_lasso_input(&self) -> Option<&dyn RouteToolLassoInput> {
        self.active_tool().and_then(|tool| tool.as_lasso_input())
    }

    /// Liefert die mutable Lasso-Capability des aktiven Tools, falls vorhanden.
    pub fn active_lasso_input_mut(&mut self) -> Option<&mut dyn RouteToolLassoInput> {
        self.active_tool_mut()
            .and_then(|tool| tool.as_lasso_input_mut())
    }

    /// Liefert die Group-Edit-Capability des aktiven Tools, falls vorhanden.
    pub fn active_group_edit(&self) -> Option<&dyn RouteToolGroupEdit> {
        self.active_tool().and_then(|tool| tool.as_group_edit())
    }

    /// Liefert die mutable Group-Edit-Capability des aktiven Tools, falls vorhanden.
    pub fn active_group_edit_mut(&mut self) -> Option<&mut dyn RouteToolGroupEdit> {
        self.active_tool_mut()
            .and_then(|tool| tool.as_group_edit_mut())
    }

    /// Laedt eine geordnete Kette in das aktive Tool, falls die Capability vorhanden ist.
    pub fn load_active_chain(&mut self, chain: OrderedNodeChain) {
        if let Some(tool) = self.active_chain_input_mut() {
            tool.load_chain(chain);
        }
    }

    /// Setzt alle Tools zurueck und deaktiviert das aktive Tool.
    pub fn reset(&mut self) {
        if let Some(i) = self.active_index {
            self.tools[i].tool.reset();
        }
        self.active_index = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tool_contract::TangentSource;
    use crate::app::ui_contract::RouteToolConfigState;
    use crate::core::{
        Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap,
    };
    use glam::Vec2;

    fn make_curve_anchor_map() -> RoadMap {
        let mut map = RoadMap::new(3);

        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(10, Vec2::new(-10.0, 0.0), NodeFlag::Regular));
        map.add_connection(Connection::new(
            10,
            1,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(-10.0, 0.0),
            Vec2::new(0.0, 0.0),
        ));
        map.add_node(MapNode::new(20, Vec2::new(20.0, 0.0), NodeFlag::Regular));
        map.add_connection(Connection::new(
            2,
            20,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(10.0, 0.0),
            Vec2::new(20.0, 0.0),
        ));
        map.ensure_spatial_index();
        map
    }

    fn ordered_chain() -> OrderedNodeChain {
        OrderedNodeChain {
            positions: vec![Vec2::ZERO, Vec2::new(10.0, 0.0), Vec2::new(20.0, 0.0)],
            start_id: 1,
            end_id: 3,
            inner_ids: vec![2],
        }
    }

    fn host_context(
        direction: ConnectionDirection,
        priority: ConnectionPriority,
    ) -> ToolHostContext {
        ToolHostContext {
            direction,
            priority,
            snap_radius: 2.5,
            farmland_data: None,
            farmland_grid: None,
            background_image: None,
        }
    }

    #[test]
    fn capability_split_separates_rotation_from_segment_adjustments() {
        let mut manager = ToolManager::new();

        manager.set_active_by_id(RouteToolId::Parking);
        assert!(manager.active_rotate().is_some());
        assert!(manager.active_segment_adjustments().is_none());

        manager.set_active_by_id(RouteToolId::CurveCubic);
        assert!(manager.active_rotate().is_none());
        assert!(manager.active_segment_adjustments().is_some());
    }

    #[test]
    fn curve_capabilities_expose_tangent_menu_and_drag_targets() {
        let road_map = make_curve_anchor_map();
        let mut manager = ToolManager::new();
        manager.set_active_by_id(RouteToolId::CurveCubic);

        manager
            .active_tool_mut()
            .expect("kubische Kurve muss aktiv sein")
            .on_click(Vec2::new(0.0, 0.0), &road_map, false);
        manager
            .active_tool_mut()
            .expect("kubische Kurve muss aktiv sein")
            .on_click(Vec2::new(10.0, 0.0), &road_map, false);

        let menu = manager
            .active_tangent()
            .and_then(|tool| tool.tangent_menu_data())
            .expect("Tangenten-Menue muss ueber die Capability sichtbar sein");
        assert!(matches!(
            menu.current_start,
            TangentSource::Connection { .. }
        ));
        assert!(matches!(menu.current_end, TangentSource::Connection { .. }));
        assert!(!manager
            .active_drag()
            .expect("CurveTool muss Drag ueber Capability melden")
            .drag_targets()
            .is_empty());
    }

    #[test]
    fn load_active_chain_routes_ordered_chain_into_bypass() {
        let mut manager = ToolManager::new();
        manager.set_active_by_id(RouteToolId::Bypass);

        assert!(manager.active_chain_input().is_some());

        manager.load_active_chain(ordered_chain());

        let tool = manager.active_tool().expect("BypassTool muss aktiv sein");
        assert!(tool.is_ready());
        let RouteToolConfigState::Bypass(panel) = tool.panel_state() else {
            panic!("Bypass-Panelzustand erwartet");
        };
        assert!(panel.has_chain);
    }

    #[test]
    fn color_path_lasso_capability_is_phase_gated() {
        let mut manager = ToolManager::new();
        let road_map = RoadMap::default();
        manager.set_active_by_id(RouteToolId::ColorPath);

        assert!(manager.active_lasso_input().is_some());
        assert!(!manager
            .active_lasso_input()
            .expect("ColorPath muss Lasso-Capability bereitstellen")
            .is_lasso_input_active());

        manager
            .active_tool_mut()
            .expect("ColorPathTool muss aktiv sein")
            .on_click(Vec2::ZERO, &road_map, false);

        assert!(manager
            .active_lasso_input()
            .expect("ColorPath muss Lasso-Capability bereitstellen")
            .is_lasso_input_active());
    }

    #[test]
    fn sync_active_host_updates_direction_and_priority_on_preview() {
        let mut manager = ToolManager::new();
        let road_map = RoadMap::default();
        manager.set_active_by_id(RouteToolId::Straight);
        manager.sync_active_host(&host_context(
            ConnectionDirection::Reverse,
            ConnectionPriority::SubPriority,
        ));

        manager
            .active_tool_mut()
            .expect("StraightLineTool muss aktiv sein")
            .on_click(Vec2::ZERO, &road_map, false);

        let preview = manager
            .active_tool()
            .expect("StraightLineTool muss aktiv sein")
            .preview(Vec2::new(12.0, 0.0), &road_map);

        assert!(!preview.connection_styles.is_empty());
        assert!(preview.connection_styles.iter().all(|style| {
            *style
                == (
                    ConnectionDirection::Reverse,
                    ConnectionPriority::SubPriority,
                )
        }));
    }
}
