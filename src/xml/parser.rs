//! Parser für AutoDrive XML-Konfigurationen.

mod markers;
mod waypoints;

use crate::core::{AutoDriveMeta, MapMarker, RoadMap};
use anyhow::{bail, Context, Result};
use markers::parse_marker_id;
use quick_xml::events::Event;
use quick_xml::Reader;
use waypoints::build_nodes_and_connections;

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

    let ids = waypoints::parse_list::<u64>(&waypoint_ids, ',')
        .context("Fehler beim Parsen der ID-Liste")?;
    let xs = waypoints::parse_list::<f32>(&waypoint_x, ',')
        .context("Fehler beim Parsen der X-Koordinaten")?;
    let zs = waypoints::parse_list::<f32>(&waypoint_z, ',')
        .context("Fehler beim Parsen der Z-Koordinaten")?;
    let flags = waypoints::parse_list::<u32>(&waypoint_flags, ',')
        .context("Fehler beim Parsen der Flags")?;
    let outgoing = waypoints::parse_nested_list(&waypoint_out)
        .context("Fehler beim Parsen der Outgoing-Liste")?;
    let incoming = waypoints::parse_nested_list(&waypoint_incoming)
        .context("Fehler beim Parsen der Incoming-Liste")?;

    let mut ys: Option<Vec<f32>> = None;
    if !waypoint_y.is_empty() {
        ys = Some(waypoints::parse_list::<f32>(&waypoint_y, ',')?);
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

    let y_str = if !waypoint_y.is_empty() {
        Some(waypoint_y.as_str())
    } else {
        None
    };
    let (nodes, connections) = build_nodes_and_connections(
        &waypoint_ids,
        &waypoint_x,
        y_str,
        &waypoint_z,
        &waypoint_flags,
        &waypoint_out,
        &waypoint_incoming,
    )?;

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
mod tests;
