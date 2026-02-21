//! Parser für AutoDrive XML-Konfigurationen.

use crate::core::{AutoDriveMeta, Connection, ConnectionDirection, ConnectionPriority, MapMarker};
use crate::core::{MapNode, NodeFlag, RoadMap};
use anyhow::bail;
use anyhow::{Context, Result};
use glam::Vec2;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

/// Parsed eine AutoDrive-Config aus einem XML-String
pub fn parse_autodrive_config(xml_content: &str) -> Result<RoadMap> {
    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);

    let mut buffer = Vec::new();

    let mut version_text: Option<String> = None;
    let mut version_attr: Option<String> = None;
    let mut map_name: Option<String> = None;
    let mut route_version: Option<String> = None;
    let mut route_author: Option<String> = None;
    let mut config_version: Option<String> = None;
    let mut options: Vec<(String, String)> = Vec::new();

    let mut in_waypoints = false;
    let mut in_mapmarker = false;
    let mut in_marker_element = false;
    let mut current_tag: Option<String> = None;

    let mut waypoint_ids = String::new();
    let mut waypoint_x = String::new();
    let mut waypoint_y = String::new();
    let mut waypoint_z = String::new();
    let mut waypoint_out = String::new();
    let mut waypoint_incoming = String::new();
    let mut waypoint_flags = String::new();

    let mut map_markers: Vec<MapMarker> = Vec::new();
    let mut marker_index = 1u32;
    let mut current_marker_tag: Option<String> = None;
    let mut current_marker_id: Option<u64> = None;
    let mut current_marker_name: Option<String> = None;
    let mut current_marker_group: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                let tag = reader.decoder().decode(name.as_ref())?;

                if tag == "AutoDrive" {
                    for attr in e.attributes().with_checks(false) {
                        let attr = attr?;
                        let key = reader.decoder().decode(attr.key.as_ref())?;
                        if key == "version" {
                            let value = attr.unescape_value()?.into_owned();
                            version_attr = Some(value);
                        }
                    }
                } else if tag == "waypoints" {
                    in_waypoints = true;
                } else if tag == "mapmarker" {
                    in_mapmarker = true;
                } else if in_mapmarker && tag.starts_with("mm") {
                    // Marker-Element beginnt (z.B. <mm1>, <mm2>, ...)
                    in_marker_element = true;
                    current_marker_tag = Some(tag.to_string());
                    current_marker_id = None;
                    current_marker_name = None;
                    current_marker_group = None;
                } else if in_waypoints {
                    current_tag = Some(tag.to_string());
                } else if in_marker_element {
                    // Innerhalb eines Marker-Elements: <id>, <name>, <group>
                    current_tag = Some(tag.to_string());
                } else if tag != "AutoDrive" {
                    current_tag = Some(tag.to_string());
                }
            }
            Ok(Event::Empty(ref e)) => {
                // Leere Tags werden ignoriert
                let _name = e.name();
            }
            Ok(Event::Text(e)) => {
                let text = e.xml_content()?.into_owned();

                if in_waypoints {
                    match current_tag.as_deref() {
                        Some("id") => waypoint_ids.push_str(&text),
                        Some("x") => waypoint_x.push_str(&text),
                        Some("y") => waypoint_y.push_str(&text),
                        Some("z") => waypoint_z.push_str(&text),
                        Some("out") => waypoint_out.push_str(&text),
                        Some("incoming") => waypoint_incoming.push_str(&text),
                        Some("flags") => waypoint_flags.push_str(&text),
                        _ => {}
                    }
                } else if in_marker_element {
                    // Text innerhalb eines Marker-Elements
                    match current_tag.as_deref() {
                        Some("id") => {
                            let marker_tag = current_marker_tag.as_deref().unwrap_or("<unknown>");
                            let id = parse_marker_id(&text).with_context(|| {
                                format!("Ungueltige Marker-ID in {}: '{}'", marker_tag, text)
                            })?;
                            current_marker_id = Some(id);
                        }
                        Some("name") => current_marker_name = Some(text),
                        Some("group") => current_marker_group = Some(text),
                        _ => {}
                    }
                } else {
                    match current_tag.as_deref() {
                        Some("version") => {
                            version_text = Some(text.clone());
                            config_version = Some(text);
                        }
                        Some("MapName") => map_name = Some(text),
                        Some("ADRouteVersion") => route_version = Some(text),
                        Some("ADRouteAuthor") => route_author = Some(text),
                        Some(tag_name) => {
                            options.push((tag_name.to_string(), text));
                        }
                        None => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                let tag = reader.decoder().decode(name.as_ref())?;
                if tag == "waypoints" {
                    in_waypoints = false;
                } else if tag == "mapmarker" {
                    in_mapmarker = false;
                } else if in_marker_element && tag.starts_with("mm") {
                    // Marker-Element endet - füge Marker hinzu
                    in_marker_element = false;
                    current_marker_tag = None;
                    if let Some(id) = current_marker_id {
                        let name = current_marker_name
                            .take()
                            .unwrap_or_else(|| "Unnamed".to_string());
                        let group = current_marker_group
                            .take()
                            .unwrap_or_else(|| "All".to_string());
                        map_markers.push(MapMarker::new(id, name, group, marker_index, false));
                        marker_index += 1;
                    }
                } else if current_tag.as_deref() == Some(tag.as_ref()) {
                    current_tag = None;
                }
            }
            Ok(Event::Eof) => break,
            Err(err) => return Err(err).context("Fehler beim Parsen des XML"),
            _ => {}
        }

        buffer.clear();
    }

    let version = parse_version(version_attr.clone(), version_text)?;

    // config_version für den Writer sicherstellen (Roundtrip-Fähigkeit)
    if config_version.is_none() {
        config_version = version_attr;
    }

    if waypoint_ids.is_empty()
        || waypoint_x.is_empty()
        || waypoint_z.is_empty()
        || waypoint_out.is_empty()
        || waypoint_incoming.is_empty()
        || waypoint_flags.is_empty()
    {
        bail!("Pflichtfelder in <waypoints> fehlen");
    }

    let ids = parse_list::<u64>(&waypoint_ids, ',').context("Fehler beim Parsen der ID-Liste")?;
    let xs = parse_list::<f32>(&waypoint_x, ',').context("Fehler beim Parsen der X-Koordinaten")?;
    let zs = parse_list::<f32>(&waypoint_z, ',').context("Fehler beim Parsen der Z-Koordinaten")?;
    let flags = parse_list::<u32>(&waypoint_flags, ',').context("Fehler beim Parsen der Flags")?;
    let outgoing =
        parse_nested_list(&waypoint_out).context("Fehler beim Parsen der Outgoing-Liste")?;
    let incoming =
        parse_nested_list(&waypoint_incoming).context("Fehler beim Parsen der Incoming-Liste")?;

    let mut ys: Option<Vec<f32>> = None;
    if !waypoint_y.is_empty() {
        ys = Some(parse_list::<f32>(&waypoint_y, ',')?);
    }

    let expected_len = ids.len();
    if xs.len() != expected_len
        || zs.len() != expected_len
        || flags.len() != expected_len
        || outgoing.len() != expected_len
        || incoming.len() != expected_len
    {
        bail!("Laengen der Waypoint-Listen stimmen nicht ueberein");
    }

    if let Some(ref ys) = ys {
        if ys.len() != expected_len {
            bail!("Laenge der y-Liste stimmt nicht ueberein");
        }
    }

    let mut nodes = HashMap::new();
    let mut id_to_index = HashMap::new();

    for (index, id) in ids.iter().enumerate() {
        let flag = NodeFlag::from_u32(flags[index]);
        let position = Vec2::new(xs[index], zs[index]);
        nodes.insert(*id, MapNode::new(*id, position, flag));
        id_to_index.insert(*id, index);
    }

    let mut connections = Vec::new();
    // Bereits angelegte Dual-Paare verfolgen, damit A→B Dual nicht auch B→A Dual erzeugt
    let mut dual_pairs: std::collections::HashSet<(u64, u64)> = std::collections::HashSet::new();

    for (index, source_id) in ids.iter().enumerate() {
        let targets = &outgoing[index];

        for target_id in targets {
            if target_id == source_id {
                continue;
            }

            let target_index = match id_to_index.get(target_id) {
                Some(idx) => *idx,
                None => {
                    log::warn!("Missing target node: {}", target_id);
                    continue;
                }
            };

            let target_out = &outgoing[target_index];
            let target_incoming = &incoming[target_index];

            let direction = if target_out.contains(source_id) {
                ConnectionDirection::Dual
            } else if !target_incoming.contains(source_id) {
                ConnectionDirection::Reverse
            } else {
                ConnectionDirection::Regular
            };

            // Bei Dual: nur einmal pro Paar anlegen (kleinere ID → größere ID)
            if direction == ConnectionDirection::Dual {
                let pair = ((*source_id).min(*target_id), (*source_id).max(*target_id));
                if dual_pairs.contains(&pair) {
                    continue;
                }
                dual_pairs.insert(pair);
            }

            let priority = match nodes.get(target_id).map(|node| node.flag) {
                Some(NodeFlag::SubPrio) => ConnectionPriority::SubPriority,
                _ => ConnectionPriority::Regular,
            };

            let start_pos = nodes.get(source_id).context("Start-Node fehlt")?.position;
            let end_pos = nodes.get(target_id).context("End-Node fehlt")?.position;

            connections.push(Connection::new(
                *source_id, *target_id, direction, priority, start_pos, end_pos,
            ));
        }
    }

    let mut road_map = RoadMap::new(version);
    road_map.map_name = map_name;
    road_map.nodes = nodes;
    for conn in connections {
        road_map.add_connection(conn);
    }
    road_map.map_markers = map_markers;
    road_map.meta = AutoDriveMeta {
        config_version,
        route_version,
        route_author,
        options,
    };
    road_map.rebuild_spatial_index();

    Ok(road_map)
}

fn parse_marker_id(text: &str) -> Result<u64> {
    let value = text
        .trim()
        .parse::<f64>()
        .context("Marker-ID ist keine gueltige Zahl")?;

    if !value.is_finite() {
        bail!("Marker-ID muss endlich sein");
    }

    if value < 0.0 {
        bail!("Marker-ID darf nicht negativ sein");
    }

    if value.fract() != 0.0 {
        bail!("Marker-ID muss ganzzahlig sein");
    }

    Ok(value as u64)
}

/// Hilfsfunktion zum Parsen einer kommagetrennten Liste
fn parse_list<T: std::str::FromStr>(text: &str, delimiter: char) -> Result<Vec<T>>
where
    <T as std::str::FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    text.split(delimiter)
        .filter(|s| !s.is_empty())
        .map(|s| {
            let trimmed = s.trim();
            trimmed.parse::<T>().with_context(|| {
                format!(
                    "Wert '{}' konnte nicht geparst werden",
                    truncate_for_error(trimmed)
                )
            })
        })
        .collect::<Result<Vec<T>, _>>()
}

/// Kürzt einen String für Fehlermeldungen auf max. 40 Zeichen
fn truncate_for_error(s: &str) -> &str {
    if s.len() <= 40 {
        s
    } else {
        &s[..40]
    }
}

/// Hilfsfunktion zum Parsen verschachtelter Listen (für out/incoming).
/// Werte <= 0 (z.B. -1) werden ignoriert — sie markieren Endpunkte oder
/// rückwärts befahrene Strecken in AutoDrive.
fn parse_nested_list(text: &str) -> Result<Vec<Vec<u64>>> {
    text.split(';')
        .map(|part| {
            if part.trim().is_empty() {
                Ok(Vec::new())
            } else {
                part.split(',')
                    .filter(|s| !s.is_empty())
                    .filter_map(|s| {
                        let trimmed = s.trim();
                        // -1 (und andere negative Werte) = kein Ziel / Endpunkt
                        if trimmed.starts_with('-') {
                            None
                        } else {
                            Some(trimmed.parse::<u64>().with_context(|| {
                                format!(
                                    "Wert '{}' konnte nicht geparst werden",
                                    truncate_for_error(trimmed)
                                )
                            }))
                        }
                    })
                    .collect()
            }
        })
        .collect()
}

fn parse_version(version_attr: Option<String>, version_text: Option<String>) -> Result<u32> {
    let value = version_attr
        .or(version_text)
        .context("Keine Version in der XML gefunden")?;

    let major = value.split('.').next().unwrap_or(&value).trim();

    let version = major
        .parse::<u32>()
        .context("Version konnte nicht gelesen werden")?;

    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_list() {
        let result = parse_list::<u64>("1,2,3,4", ',').unwrap();
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_parse_nested_list() {
        let result = parse_nested_list("2,3;4,5;;1").unwrap();
        assert_eq!(result, vec![vec![2, 3], vec![4, 5], vec![], vec![1],]);
    }

    #[test]
    fn test_parse_fails_for_invalid_marker_id() {
        let xml = r#"
        <AutoDrive version="3">
            <waypoints>
                <id>1</id>
                <x>0</x>
                <y>0</y>
                <z>0</z>
                <out></out>
                <incoming></incoming>
                <flags>0</flags>
            </waypoints>
            <mapmarker>
                <mm1>
                    <id>abc</id>
                    <name>Test</name>
                    <group>All</group>
                </mm1>
            </mapmarker>
        </AutoDrive>
        "#;

        let err = parse_autodrive_config(xml).expect_err("Parser sollte fehlschlagen");
        let msg = format!("{err:#}");
        assert!(msg.contains("Ungueltige Marker-ID"));
    }

    #[test]
    fn test_bidirectional_creates_single_connection() {
        // Node 1 ↔ Node 2 (beide in out des jeweils anderen)
        let xml = r#"
        <AutoDrive version="3">
            <waypoints>
                <id>1,2</id>
                <x>0,10</x>
                <y>0,0</y>
                <z>0,0</z>
                <out>2;1</out>
                <incoming>2;1</incoming>
                <flags>0,0</flags>
            </waypoints>
            <mapmarker></mapmarker>
        </AutoDrive>
        "#;

        let road_map = parse_autodrive_config(xml).expect("Parsing fehlgeschlagen");
        assert_eq!(
            road_map.connection_count(),
            1,
            "Bidirektional soll nur 1 Connection erzeugen"
        );

        let conn = road_map.connections_iter().next().expect("Connection erwartet");
        assert_eq!(conn.direction, ConnectionDirection::Dual);
    }

    #[test]
    fn test_bidirectional_roundtrip_preserves_connections() {
        use crate::xml::writer::write_autodrive_config;

        // 3 Nodes: 1 ↔ 2 (dual), 2 → 3 (regular)
        let xml = r#"
        <AutoDrive version="3">
            <waypoints>
                <id>1,2,3</id>
                <x>0,10,20</x>
                <y>0,0,0</y>
                <z>0,0,0</z>
                <out>2;1,3;-1</out>
                <incoming>2;1;2</incoming>
                <flags>0,0,0</flags>
            </waypoints>
            <mapmarker></mapmarker>
        </AutoDrive>
        "#;

        let road_map = parse_autodrive_config(xml).expect("Parsing fehlgeschlagen");
        assert_eq!(
            road_map.connection_count(),
            2,
            "1 Dual + 1 Regular = 2 Connections"
        );

        let written = write_autodrive_config(&road_map, None).expect("Export fehlgeschlagen");
        let reparsed = parse_autodrive_config(&written).expect("Re-Parsing fehlgeschlagen");
        assert_eq!(reparsed.connection_count(), road_map.connection_count());
    }
}
