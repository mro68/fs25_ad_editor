//! Writer für AutoDrive XML-Konfigurationen.

use crate::core::{ConnectionDirection, Heightmap, RoadMap};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

/// Schreibt eine RoadMap als AutoDrive XML-Config
///
/// # Parameter
/// - `road_map`: Die zu exportierende RoadMap
/// - `heightmap`: Optionale Heightmap für Y-Koordinaten-Berechnung
pub fn write_autodrive_config(road_map: &RoadMap, heightmap: Option<&Heightmap>) -> Result<String> {
    let mut output = String::new();
    output.push_str("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"no\"?>\n");
    output.push_str("<AutoDrive>\n");

    if let Some(ref config_version) = road_map.meta.config_version {
        output.push_str(&format!(
            "    <version>{}</version>\n",
            escape_xml(config_version)
        ));
    }

    if let Some(ref map_name) = road_map.map_name {
        output.push_str(&format!(
            "    <MapName>{}</MapName>\n",
            escape_xml(map_name)
        ));
    }

    if let Some(ref route_version) = road_map.meta.route_version {
        output.push_str(&format!(
            "    <ADRouteVersion>{}</ADRouteVersion>\n",
            escape_xml(route_version)
        ));
    }

    if let Some(ref route_author) = road_map.meta.route_author {
        output.push_str(&format!(
            "    <ADRouteAuthor>{}</ADRouteAuthor>\n",
            escape_xml(route_author)
        ));
    }

    // Options in Original-Reihenfolge schreiben
    for (key, value) in &road_map.meta.options {
        output.push_str(&format!("    <{}>{}</{}>\n", key, escape_xml(value), key));
    }

    let mut node_ids: Vec<u64> = road_map.nodes.keys().copied().collect();
    node_ids.sort_unstable();

    // Renumbering: Interne IDs → lückenlose 1-basierte IDs (AutoDrive erwartet kontiguöse IDs)
    let id_remap: HashMap<u64, u64> = node_ids
        .iter()
        .enumerate()
        .map(|(i, &old_id)| (old_id, (i + 1) as u64))
        .collect();

    let mut outgoing: HashMap<u64, HashSet<u64>> = HashMap::new();
    let mut incoming: HashMap<u64, HashSet<u64>> = HashMap::new();

    for id in &node_ids {
        outgoing.insert(*id, HashSet::new());
        incoming.insert(*id, HashSet::new());
    }

    for connection in road_map.connections_iter() {
        let start_id = connection.start_id;
        let end_id = connection.end_id;

        if let Some(list) = outgoing.get_mut(&start_id) {
            list.insert(end_id);
        }

        if connection.direction != ConnectionDirection::Reverse {
            if let Some(list) = incoming.get_mut(&end_id) {
                list.insert(start_id);
            }
        }

        if connection.direction == ConnectionDirection::Dual {
            if let Some(list) = outgoing.get_mut(&end_id) {
                list.insert(start_id);
            }

            if let Some(list) = incoming.get_mut(&start_id) {
                list.insert(end_id);
            }
        }
    }

    let mut ids_text = Vec::new();
    let mut xs_text = Vec::new();
    let mut ys_text = Vec::new();
    let mut zs_text = Vec::new();
    let mut flags_text = Vec::new();
    let mut out_text = Vec::new();
    let mut incoming_text = Vec::new();

    for id in &node_ids {
        let node = road_map.nodes.get(id).ok_or_else(|| {
            anyhow::anyhow!("Inkonsistente RoadMap: Node {} fehlt beim XML-Export", id)
        })?;
        let new_id = id_remap[id];
        ids_text.push(new_id.to_string());
        xs_text.push(format_float(node.position.x));

        // Y-Koordinate: Aus Heightmap berechnen oder 0.0
        let y_value = if let Some(hm) = heightmap {
            // FS25: Y = normalized_pixel × 255.0 (Standard-Terrainhöhe)
            let height = hm.sample_height(node.position.x, node.position.y, 255.0);

            // Debug: Zeige erste 10 Y-Werte zur Kontrolle
            if ids_text.len() <= 10 {
                log::info!(
                    "Node {}: pos=({:.3}, {:.3}) -> Y={:.3}m",
                    id,
                    node.position.x,
                    node.position.y,
                    height
                );
            }

            height
        } else {
            0.0
        };
        ys_text.push(format_float(y_value));

        zs_text.push(format_float(node.position.y));
        flags_text.push(node.flag.to_u32().to_string());

        let mut out_list: Vec<u64> = outgoing
            .get(id)
            .map(|list| list.iter().filter_map(|old| id_remap.get(old).copied()).collect())
            .unwrap_or_default();
        out_list.sort_unstable();
        out_text.push(join_ids(&out_list));

        let mut incoming_list: Vec<u64> = incoming
            .get(id)
            .map(|list| list.iter().filter_map(|old| id_remap.get(old).copied()).collect())
            .unwrap_or_default();
        incoming_list.sort_unstable();
        incoming_text.push(join_ids(&incoming_list));
    }

    output.push_str("    <waypoints>\n");
    output.push_str(&format!("        <id>{}</id>\n", ids_text.join(",")));
    output.push_str(&format!("        <x>{}</x>\n", xs_text.join(",")));
    output.push_str(&format!("        <y>{}</y>\n", ys_text.join(",")));
    output.push_str(&format!("        <z>{}</z>\n", zs_text.join(",")));
    output.push_str(&format!("        <out>{}</out>\n", out_text.join(";")));
    output.push_str(&format!(
        "        <incoming>{}</incoming>\n",
        incoming_text.join(";")
    ));
    output.push_str(&format!(
        "        <flags>{}</flags>\n",
        flags_text.join(",")
    ));
    output.push_str("    </waypoints>\n");

    output.push_str("    <mapmarker>\n");
    for (index, marker) in road_map.map_markers.iter().enumerate() {
        let marker_tag = format!("mm{}", index + 1);
        // Marker-ID remappen (zeigt auf Node-ID)
        let remapped_marker_id = id_remap.get(&marker.id).copied().unwrap_or(marker.id);
        output.push_str(&format!("        <{}>\n", marker_tag));
        output.push_str(&format!(
            "            <id>{:.6}</id>\n",
            remapped_marker_id as f64
        ));
        output.push_str(&format!(
            "            <name>{}</name>\n",
            escape_xml(&marker.name)
        ));
        output.push_str(&format!(
            "            <group>{}</group>\n",
            escape_xml(&marker.group)
        ));
        output.push_str(&format!("        </{}>\n", marker_tag));
    }
    output.push_str("    </mapmarker>\n");

    output.push_str("</AutoDrive>\n");

    Ok(output)
}

fn join_ids(ids: &[u64]) -> String {
    ids.iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(",")
}

fn format_float(value: f32) -> String {
    format!("{:.3}", value)
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_float_precision() {
        // Testet, dass Koordinaten auf 3 Dezimalstellen gerundet werden
        assert_eq!(format_float(123.456_79), "123.457");
        assert_eq!(format_float(100.0), "100.000");
        assert_eq!(format_float(0.001_234_56), "0.001");
        assert_eq!(format_float(-50.123_456), "-50.123");
        assert_eq!(format_float(1_234.999_9), "1235.000");
    }
}
