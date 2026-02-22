//! Wegpunkt-Parsing: Konvertiert rohe String-Puffer in Nodes und Connections.

use crate::core::{Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag};
use anyhow::{bail, Context, Result};
use glam::Vec2;
use std::collections::HashMap;

/// Baut Nodes und Connections aus den geparsten Waypoint-Rohdaten auf.
///
/// Die Parameter entsprechen den komma- bzw. semikolon-getrennten Strings aus
/// dem `<waypoints>`-Block des AutoDrive-XML.
pub(super) fn build_nodes_and_connections(
    ids_raw: &str,
    x_raw: &str,
    y_raw: Option<&str>,
    z_raw: &str,
    flags_raw: &str,
    out_raw: &str,
    incoming_raw: &str,
) -> Result<(HashMap<u64, MapNode>, Vec<Connection>)> {
    let ids = parse_list::<u64>(ids_raw, ',').context("Fehler beim Parsen der ID-Liste")?;
    let xs = parse_list::<f32>(x_raw, ',').context("Fehler beim Parsen der X-Koordinaten")?;
    let zs = parse_list::<f32>(z_raw, ',').context("Fehler beim Parsen der Z-Koordinaten")?;
    let flags = parse_list::<u32>(flags_raw, ',').context("Fehler beim Parsen der Flags")?;
    let outgoing = parse_nested_list(out_raw).context("Fehler beim Parsen der Outgoing-Liste")?;
    let incoming =
        parse_nested_list(incoming_raw).context("Fehler beim Parsen der Incoming-Liste")?;

    let ys: Option<Vec<f32>> = if let Some(y) = y_raw {
        if !y.is_empty() {
            Some(parse_list::<f32>(y, ',')?)
        } else {
            None
        }
    } else {
        None
    };

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

    // Phase 1: Nodes aufbauen
    let mut nodes = HashMap::new();
    let mut id_to_index = HashMap::new();

    for (index, id) in ids.iter().enumerate() {
        let flag = NodeFlag::from_u32(flags[index]);
        let position = Vec2::new(xs[index], zs[index]);
        nodes.insert(*id, MapNode::new(*id, position, flag));
        id_to_index.insert(*id, index);
    }

    // Phase 2: Connections aufbauen
    let mut connections = Vec::new();
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

            // Bei Dual: nur einmal pro Paar anlegen
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

    Ok((nodes, connections))
}

/// Parst eine kommagetrennte Liste einfacher Werte.
pub(super) fn parse_list<T: std::str::FromStr>(text: &str, delimiter: char) -> Result<Vec<T>>
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

/// Parst verschachtelte Listen (für out/incoming).
///
/// Werte ≤ 0 (z.B. -1) werden ignoriert — sie markieren Endpunkte oder
/// rückwärts befahrene Strecken in AutoDrive.
pub(super) fn parse_nested_list(text: &str) -> Result<Vec<Vec<u64>>> {
    text.split(';')
        .map(|part| {
            if part.trim().is_empty() {
                Ok(Vec::new())
            } else {
                part.split(',')
                    .filter(|s| !s.is_empty())
                    .filter_map(|s| {
                        let trimmed = s.trim();
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

/// Kürzt einen String für Fehlermeldungen auf max. 40 Zeichen.
fn truncate_for_error(s: &str) -> &str {
    if s.len() <= 40 {
        s
    } else {
        &s[..40]
    }
}
