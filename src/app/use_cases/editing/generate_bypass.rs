//! Use-Case: Ausweichstrecke parallel zur selektierten Kette generieren.
//!
//! Erzeugt eine versetzte Parallelstrecke zwischen den Endpunkten der selektierten
//! Kette mit S-förmigen An- und Abfahrtsbögen.
//!
//! # Geometrie
//!
//! ```text
//!  chain[0] ──S-Entry──▶ b0 ──── Hauptstrecke (verkürzt) ────▶ bn ──S-Exit──▶ chain[n-1]
//! ```
//!
//! Die Hauptstrecke beginnt und endet nicht bei `chain[0]`/`chain[n-1]`, sondern
//! jeweils `d_blend = |offset| × 1.5` entlang der Kette versetzt. Dadurch haben die
//! S-Kurven ausreichend Längsraum für tangentiale An-/Abfahrten.
//!
//! - **S-Kurven**: Kubische Bézier-Kurven, halber Knotenabstand (`base_spacing * 0.5`)
//! - **Hauptstrecke**: Parallel-Offset des mittleren Kettenabschnitts, voller Knotenabstand
//! - **Verbindungsrichtung**: Von selektiertem Start zu selektiertem Ende

use crate::app::AppState;
use crate::core::{Connection, MapNode, NodeFlag};
use crate::shared::spline_geometry::{
    catmull_rom_chain_with_tangents, polyline_length, resample_by_distance,
};
use glam::Vec2;
use std::sync::Arc;

// ─── öffentliche Einstiegsfunktion ────────────────────────────────────────────

/// Generiert eine Ausweichstrecke parallel zur selektierten Kette und fügt sie in
/// die RoadMap ein. Die neuen Nodes werden nach Abschluss selektiert.
///
/// Liest aus `state.ui.bypass`:
/// - `offset`       — seitlicher Versatz (positiv = links)
/// - `base_spacing` — Knotenabstand auf der Hauptstrecke
///
/// Gibt silently zurück, wenn weniger als 2 Nodes selektiert sind oder die
/// Selektion keine lineare Kette bildet.
pub fn generate_bypass(state: &mut AppState) {
    // ── Vorbedingungen ────────────────────────────────────────────────────────
    let Some(road_map_ref) = state.road_map.as_ref() else {
        log::warn!("Ausweichstrecke: keine RoadMap geladen");
        return;
    };

    let n_sel = state.selection.selected_node_ids.len();
    if n_sel < 2 {
        log::warn!("Ausweichstrecke: mindestens 2 Nodes selektieren");
        return;
    }

    let offset = state.ui.bypass.offset;
    let base_spacing = state.ui.bypass.base_spacing.max(0.5);
    let half_spacing = base_spacing * 0.5;

    // ── Kette ordnen ─────────────────────────────────────────────────────────
    let selected = state.selection.selected_node_ids.clone();
    let Some(ordered_ids) = road_map_ref.ordered_chain_nodes(&selected) else {
        log::warn!("Ausweichstrecke: selektierte Nodes bilden keine lineare Kette");
        return;
    };

    let positions: Vec<Vec2> = ordered_ids
        .iter()
        .filter_map(|id| road_map_ref.nodes.get(id).map(|n| n.position))
        .collect();

    if positions.len() < 2 {
        return;
    }

    let chain_start_id = *ordered_ids.first().unwrap();
    let chain_end_id = *ordered_ids.last().unwrap();

    // ── Verbindungsparameter aus erster Ketten-Verbindung ─────────────────────
    let (direction, priority) = road_map_ref
        .find_connection(ordered_ids[0], ordered_ids[1])
        .map(|c| (c.direction, c.priority))
        .unwrap_or((
            state.editor.default_direction,
            state.editor.default_priority,
        ));

    // ── Catmull-Rom-Dichte-Spline ─────────────────────────────────────────────
    const SAMPLES: usize = 20;
    let dense = catmull_rom_chain_with_tangents(&positions, SAMPLES, None, None);
    let total_len = polyline_length(&dense);

    if total_len < f32::EPSILON {
        return;
    }

    // ── Übergangslänge entlang der Kette ─────────────────────────────────────
    // d_blend: wie weit die S-Kurve entlang der Originalkette läuft, bevor sie
    // auf den Offset-Punkt trifft. Mindestens |offset|, maximal 35 % der Länge.
    let d_blend = (offset.abs() * 1.5).clamp(offset.abs().max(0.1), total_len * 0.35);

    // ── Exakte Blend-Punkte auf der Dichte-Spline ─────────────────────────────
    // (b0_chain = Punkt auf Originalkette bei d_blend, bn_chain = bei total_len − d_blend)
    let Some((i0, t0)) = arc_split(&dense, d_blend) else {
        log::warn!("Ausweichstrecke: d_blend übersteigt Kettenläng (Entry)");
        return;
    };
    let Some((i_n, t_n)) = arc_split(&dense, total_len - d_blend) else {
        log::warn!("Ausweichstrecke: d_blend übersteigt Kettenlänge (Exit)");
        return;
    };

    // Exakte Positionen und lokale Tangenten am Blend-Punkt
    let b0_chain = dense[i0].lerp(dense[i0 + 1], t0);
    let t_at_b0 = (dense[i0 + 1] - dense[i0]).normalize_or_zero();
    let perp_at_b0 = Vec2::new(-t_at_b0.y, t_at_b0.x);
    let b0 = b0_chain + perp_at_b0 * offset;

    let bn_chain = dense[i_n].lerp(dense[i_n + 1], t_n);
    let t_at_bn = (dense[i_n + 1] - dense[i_n]).normalize_or_zero();
    let perp_at_bn = Vec2::new(-t_at_bn.y, t_at_bn.x);
    let bn = bn_chain + perp_at_bn * offset;

    // ── Mittleres Teilstück der Dichte-Spline für die Hauptstrecke ───────────
    // Wir nutzen nur die Punkte zwischen i0 und i_n (plus exakte Endpunkte).
    let mut main_chain: Vec<Vec2> = Vec::with_capacity(i_n - i0 + 3);
    main_chain.push(b0_chain);
    for j in (i0 + 1)..=i_n {
        main_chain.push(dense[j]);
    }
    main_chain.push(bn_chain);

    // Parallel-Offset der Hauptstrecke (erste/letzte Punkte entsprechen b0/bn exakt)
    let main_offset = parallel_offset(&main_chain, offset);
    let main_pts = resample_by_distance(&main_offset, base_spacing);

    // ── S-Kurven: Bézier mit tangentialen Kontrollpunkten ────────────────────
    // Kettenanfangs- und -endtangente
    let t_chain_start = (dense[1] - dense[0]).normalize_or_zero();
    let t_chain_end = (*dense.last().unwrap() - dense[dense.len() - 2]).normalize_or_zero();

    // Kontrollpunkt-Abstand = d_blend × 0.45 (empfohlener Faktor für kubische Bézier)
    const CP: f32 = 0.45;
    let cp_dist = d_blend * CP;

    // S-Entry: chain[0] → b0
    // CP1 zeigt vorwärts entlang der Kette, CP2 zeigt rückwärts entlang Kette bei b0
    let entry_pts = sample_bezier(
        positions[0],
        positions[0] + t_chain_start * cp_dist,
        b0 - t_at_b0 * cp_dist,
        b0,
        half_spacing,
    );

    // S-Exit: bn → chain[n-1]
    // CP1 zeigt vorwärts ab bn (entlang Kettentangente), CP2 zeigt rückwärts am Kettenende
    let exit_pts = sample_bezier(
        bn,
        bn + t_at_bn * cp_dist,
        *positions.last().unwrap() - t_chain_end * cp_dist,
        *positions.last().unwrap(),
        half_spacing,
    );

    // ── Neue Knoten-Positionen zusammenstellen ─────────────────────────────────
    // entry_pts : [chain[0], ..., b0] → chain[0] überspringen (existiert bereits)
    // main_pts  : [b0, ..., bn]       → b0 überspringen (bereits in entry_pts)
    // exit_pts  : [bn, ..., chain[n-1]] → bn überspringen, chain[n-1] überspringen
    let entry_new: Vec<Vec2> = entry_pts.iter().skip(1).copied().collect();
    let main_new: Vec<Vec2> = main_pts.iter().skip(1).copied().collect();
    let exit_new: Vec<Vec2> = exit_pts
        .iter()
        .skip(1) // bn bereits in main_new
        .take(exit_pts.len().saturating_sub(2)) // chain[n-1] existiert bereits
        .copied()
        .collect();

    let all_new_positions: Vec<Vec2> = entry_new
        .iter()
        .chain(main_new.iter())
        .chain(exit_new.iter())
        .copied()
        .collect();

    if all_new_positions.is_empty() {
        log::warn!("Ausweichstrecke: Keine Knoten erzeugt (Abstand zu groß?)");
        return;
    }

    // ── Undo-Snapshot ─────────────────────────────────────────────────────────
    state.record_undo_snapshot();

    let road_map = Arc::make_mut(state.road_map.as_mut().unwrap());

    // ── Nodes anlegen ─────────────────────────────────────────────────────────
    let mut new_ids: Vec<u64> = Vec::with_capacity(all_new_positions.len());
    for &pos in &all_new_positions {
        let id = road_map.next_node_id();
        road_map.add_node(MapNode::new(id, pos, NodeFlag::Regular));
        new_ids.push(id);
    }

    // ── Verbindungen erstellen ─────────────────────────────────────────────────
    // 1. chain_start → first new node
    {
        let from_pos = road_map.nodes.get(&chain_start_id).unwrap().position;
        let to_pos = road_map.nodes.get(&new_ids[0]).unwrap().position;
        road_map.add_connection(Connection::new(
            chain_start_id,
            new_ids[0],
            direction,
            priority,
            from_pos,
            to_pos,
        ));
    }

    // 2. Intern: new[i] → new[i+1]
    for i in 0..new_ids.len().saturating_sub(1) {
        let from_id = new_ids[i];
        let to_id = new_ids[i + 1];
        let from_pos = road_map.nodes.get(&from_id).unwrap().position;
        let to_pos = road_map.nodes.get(&to_id).unwrap().position;
        road_map.add_connection(Connection::new(
            from_id, to_id, direction, priority, from_pos, to_pos,
        ));
    }

    // 3. last new node → chain_end
    {
        let from_id = *new_ids.last().unwrap();
        let from_pos = road_map.nodes.get(&from_id).unwrap().position;
        let to_pos = road_map.nodes.get(&chain_end_id).unwrap().position;
        road_map.add_connection(Connection::new(
            from_id,
            chain_end_id,
            direction,
            priority,
            from_pos,
            to_pos,
        ));
    }

    // ── Flags + Spatial-Index aktualisieren ────────────────────────────────────
    road_map.recalculate_node_flags(&new_ids);
    let mut endpoint_neighbors = vec![chain_start_id, chain_end_id];
    endpoint_neighbors.extend_from_slice(&new_ids);
    road_map.recalculate_node_flags(&endpoint_neighbors);
    road_map.ensure_spatial_index();

    // ── Selektion auf neue Nodes setzen ───────────────────────────────────────
    state.selection.ids_mut().clear();
    for &id in &new_ids {
        state.selection.ids_mut().insert(id);
    }
    state.selection.selection_anchor_node_id = new_ids.first().copied();

    log::info!(
        "Ausweichstrecke erzeugt: {} neue Nodes (Offset {:.1}, d_blend {:.1}, Abstand {:.1})",
        new_ids.len(),
        offset,
        d_blend,
        base_spacing,
    );
}

// ─── private Hilfsfunktionen ──────────────────────────────────────────────────

/// Findet den Schnittpunkt einer Polyline mit einer gegebenen Arc-Distanz.
///
/// Gibt `(index, t)` zurück, sodass `poly[index].lerp(poly[index+1], t)` der exakte
/// Punkt bei Arc-Distanz `target` ist. `None` wenn `target` die Gesamtlänge übersteigt.
fn arc_split(poly: &[Vec2], target: f32) -> Option<(usize, f32)> {
    let mut acc = 0.0_f32;
    for i in 0..poly.len().saturating_sub(1) {
        let seg = poly[i].distance(poly[i + 1]);
        if acc + seg >= target {
            let t = if seg > f32::EPSILON {
                (target - acc) / seg
            } else {
                0.0
            };
            return Some((i, t.clamp(0.0, 1.0)));
        }
        acc += seg;
    }
    None
}

/// Gibt einen Punkt auf einer kubischen Bézier-Kurve bei Parameter t ∈ [0, 1] zurück.
fn cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
}

/// Berechnet Punkte auf einer kubischen Bézier-Kurve mit gegebenem maximalen Abstand.
///
/// Gibt immer p0 (Index 0) und p3 (letzter Index) zurück.
fn sample_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, step: f32) -> Vec<Vec2> {
    // Dichte Approximation für Arc-Length
    const DENSE: usize = 64;
    let dense: Vec<Vec2> = (0..=DENSE)
        .map(|i| cubic_bezier(p0, p1, p2, p3, i as f32 / DENSE as f32))
        .collect();
    resample_by_distance(&dense, step)
}

/// Berechnet einen Parallel-Offset einer Polyline.
///
/// `offset > 0` → links (positive Senkrechte), `offset < 0` → rechts.
fn parallel_offset(polyline: &[Vec2], offset: f32) -> Vec<Vec2> {
    if polyline.len() < 2 {
        return polyline.to_vec();
    }
    polyline
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            let perp = local_perp(i, polyline);
            p + perp * offset
        })
        .collect()
}

/// Lokale Senkrechte am Index i einer Polyline (Durchschnitt benachbarter Segmente).
///
/// Zeigt bei `offset > 0` nach links (im Sinne der Fahrtrichtung).
fn local_perp(i: usize, poly: &[Vec2]) -> Vec2 {
    let n = poly.len();
    let tangent = if i == 0 {
        (poly[1] - poly[0]).normalize_or_zero()
    } else if i == n - 1 {
        (poly[n - 1] - poly[n - 2]).normalize_or_zero()
    } else {
        let t1 = (poly[i] - poly[i - 1]).normalize_or_zero();
        let t2 = (poly[i + 1] - poly[i]).normalize_or_zero();
        (t1 + t2).normalize_or_zero()
    };
    // Linkssenkrechte: dreht 90° gegen den Uhrzeigersinn
    Vec2::new(-tangent.y, tangent.x)
}
