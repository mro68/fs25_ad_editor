//! Skelett-Extraktion fuer das ColorPathTool.
//!
//! Pipeline: Bool-Maske → Zhang-Suen-Thinning → Pixel-Graph →
//! Junction-Clustering → Segment-Polylines in Weltkoordinaten.

use glam::Vec2;
use std::collections::{HashMap, HashSet, VecDeque};

use super::sampling::{morphological_close, morphological_open, pixel_to_world_f32};
use crate::core::zhang_suen_thinning;

/// Mindest-Pixelanzahl einer Komponente — kuerzere Fragmente werden verworfen.
const MIN_COMPONENT_PIXELS: usize = 5;

type Pixel = (usize, usize);

#[derive(Clone, Copy)]
struct SkeletonBuildContext {
    width: usize,
    height: usize,
    map_size: f32,
    img_width: u32,
    img_height: u32,
}

impl SkeletonBuildContext {
    fn pixel_to_world(self, x: f32, y: f32) -> Vec2 {
        pixel_to_world_f32(x, y, self.map_size, self.img_width, self.img_height)
    }
}

/// Typ eines Netz-Knotens im extrahierten Skelettgraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SkeletonGraphNodeKind {
    /// Offenes Segment-Ende (Pixelgrad 1).
    OpenEnd,
    /// Verzweigung/Kreuzung (Cluster aus Pixeln mit Grad >= 3).
    Junction,
    /// Künstlicher Anker fuer geschlossene Schleifen ohne Enden/Junctions.
    LoopAnchor,
}

/// Ein Knoten des extrahierten Skelettgraphen.
#[derive(Debug, Clone)]
pub(crate) struct SkeletonGraphNode {
    /// Knoten-Typ fuer Preview-Stats und Anschluss-Modi.
    pub kind: SkeletonGraphNodeKind,
    /// Pixelposition des Knotenzentrums (fuer Tests/Analyse).
    #[allow(dead_code)]
    pub pixel_position: Vec2,
    /// Weltposition des Knotens.
    pub world_position: Vec2,
}

/// Ein Segment zwischen zwei Graph-Knoten.
#[derive(Debug, Clone)]
pub(crate) struct SkeletonGraphSegment {
    /// Start-Knotenindex in `SkeletonNetwork.nodes`.
    pub start_node: usize,
    /// End-Knotenindex in `SkeletonNetwork.nodes`.
    pub end_node: usize,
    /// Roh-Polyline in Weltkoordinaten inklusive Start/End-Knoten.
    pub polyline: Vec<Vec2>,
}

/// Vollstaendiges Teilnetz aus Knoten und Segmenten.
#[derive(Debug, Clone, Default)]
pub(crate) struct SkeletonNetwork {
    /// Alle offenen Enden, Kreuzungen und Schleifen-Anker.
    pub nodes: Vec<SkeletonGraphNode>,
    /// Alle verfolgten Segmente zwischen den Knoten.
    pub segments: Vec<SkeletonGraphSegment>,
}

impl SkeletonNetwork {
    /// Gibt `true` zurueck wenn kein exportierbares Segment vorliegt.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Anzahl der Kreuzungs-/Verzweigungsknoten.
    pub fn junction_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| node.kind == SkeletonGraphNodeKind::Junction)
            .count()
    }

    /// Anzahl der offenen Segmentenden.
    pub fn open_end_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| node.kind == SkeletonGraphNodeKind::OpenEnd)
            .count()
    }
}

// ---------------------------------------------------------------------------
// Verbundene Komponenten (Flood-Fill)
// ---------------------------------------------------------------------------

/// Findet alle zusammenhaengenden Skelett-Pixel-Gruppen (8-Connectivity).
///
/// Iteriert ueber alle `true`-Pixel der Maske und fuehrt pro Gruppe eine
/// BFS durch. Gibt die Gruppen sortiert nach Groesse zurueck (laengste zuerst).
pub(crate) fn find_connected_components(
    mask: &[bool],
    width: usize,
    height: usize,
) -> Vec<Vec<Pixel>> {
    let mut visited = vec![false; mask.len()];
    let mut components = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if !mask[idx] || visited[idx] {
                continue;
            }

            // BFS fuer diese zusammenhaengende Gruppe
            let mut component = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back((x, y));
            visited[idx] = true;

            while let Some((cx, cy)) = queue.pop_front() {
                component.push((cx, cy));
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = cx as i32 + dx;
                        let ny = cy as i32 + dy;
                        if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                            continue;
                        }
                        let nidx = ny as usize * width + nx as usize;
                        if mask[nidx] && !visited[nidx] {
                            visited[nidx] = true;
                            queue.push_back((nx as usize, ny as usize));
                        }
                    }
                }
            }
            components.push(component);
        }
    }

    // Laengste Gruppe zuerst
    components.sort_by_key(|c: &Vec<(usize, usize)>| std::cmp::Reverse(c.len()));
    components
}

// ---------------------------------------------------------------------------
// Pixel-Graph-Helfer
// ---------------------------------------------------------------------------

/// Liefert alle 8-benachbarten Skelettpixel eines Pixels.
fn skeleton_neighbors(pixel: Pixel, pixel_set: &HashSet<Pixel>) -> Vec<Pixel> {
    let (x, y) = pixel;
    let mut neighbors = Vec::with_capacity(8);
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || ny < 0 {
                continue;
            }
            let candidate = (nx as usize, ny as usize);
            if pixel_set.contains(&candidate) {
                neighbors.push(candidate);
            }
        }
    }
    neighbors
}

/// Normiert eine Pixelkante fuer ungerichtete Besuchsmarkierungen.
fn normalized_edge(a: Pixel, b: Pixel) -> (Pixel, Pixel) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Gruppiert zusammenhaengende Pixel in 8-Nachbarschafts-Clustern.
fn cluster_pixels(pixels: &HashSet<Pixel>) -> Vec<Vec<Pixel>> {
    let mut visited: HashSet<Pixel> = HashSet::new();
    let mut seeds: Vec<Pixel> = pixels.iter().copied().collect();
    seeds.sort_unstable();

    let mut clusters = Vec::new();
    for seed in seeds {
        if visited.contains(&seed) {
            continue;
        }

        let mut queue = VecDeque::new();
        let mut cluster = Vec::new();
        queue.push_back(seed);
        visited.insert(seed);

        while let Some(pixel) = queue.pop_front() {
            cluster.push(pixel);
            for neighbor in skeleton_neighbors(pixel, pixels) {
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }

        cluster.sort_unstable();
        clusters.push(cluster);
    }

    clusters
}

/// Waehlt fuer eine geschlossene Schleife einen stabilen Startpixel.
fn choose_anchor_pixel(pixels: &[Pixel], hint: Option<Pixel>) -> Pixel {
    if let Some((hx, hy)) = hint {
        pixels
            .iter()
            .copied()
            .min_by_key(|&(px, py)| {
                let dx = px as i64 - hx as i64;
                let dy = py as i64 - hy as i64;
                dx * dx + dy * dy
            })
            .unwrap_or(pixels[0])
    } else {
        pixels[0]
    }
}

/// Berechnet einen Graph-Knoten aus einem Pixel-Cluster.
fn build_graph_node(
    pixels: &[Pixel],
    kind: SkeletonGraphNodeKind,
    context: SkeletonBuildContext,
) -> SkeletonGraphNode {
    let count = pixels.len() as f32;
    let (sum_x, sum_y) = pixels.iter().fold((0.0f32, 0.0f32), |acc, &(x, y)| {
        (acc.0 + x as f32, acc.1 + y as f32)
    });
    let pixel_position = Vec2::new(sum_x / count, sum_y / count);
    let world_position = context.pixel_to_world(pixel_position.x, pixel_position.y);

    SkeletonGraphNode {
        kind,
        pixel_position,
        world_position,
    }
}

/// Interne Repräsentation der Pixel-zu-Knoten-Zuordnung einer Komponente.
struct ComponentGraph {
    nodes: Vec<SkeletonGraphNode>,
    node_pixels: Vec<Vec<Pixel>>,
    pixel_to_node: HashMap<Pixel, usize>,
}

/// Baut Kreuzungs- und Endknoten fuer eine Komponente auf.
fn build_component_graph(
    component: &[Pixel],
    degrees: &HashMap<Pixel, usize>,
    context: SkeletonBuildContext,
    start_hint: Option<Pixel>,
) -> ComponentGraph {
    let junction_pixels: HashSet<Pixel> = degrees
        .iter()
        .filter_map(|(&pixel, &degree)| (degree >= 3).then_some(pixel))
        .collect();
    let mut endpoint_pixels: Vec<Pixel> = degrees
        .iter()
        .filter_map(|(&pixel, &degree)| (degree <= 1).then_some(pixel))
        .collect();
    endpoint_pixels.sort_unstable();

    let mut nodes = Vec::new();
    let mut node_pixels = Vec::new();
    let mut pixel_to_node = HashMap::new();

    for cluster in cluster_pixels(&junction_pixels) {
        let node_index = nodes.len();
        for &pixel in &cluster {
            pixel_to_node.insert(pixel, node_index);
        }
        nodes.push(build_graph_node(
            &cluster,
            SkeletonGraphNodeKind::Junction,
            context,
        ));
        node_pixels.push(cluster);
    }

    for endpoint in endpoint_pixels {
        let node_index = nodes.len();
        pixel_to_node.insert(endpoint, node_index);
        nodes.push(build_graph_node(
            &[endpoint],
            SkeletonGraphNodeKind::OpenEnd,
            context,
        ));
        node_pixels.push(vec![endpoint]);
    }

    // Geschlossene Schleifen haben nur Grad-2-Pixel und brauchen einen kuenstlichen Anker.
    if nodes.is_empty() {
        let anchor = choose_anchor_pixel(component, start_hint);
        pixel_to_node.insert(anchor, 0);
        nodes.push(build_graph_node(
            &[anchor],
            SkeletonGraphNodeKind::LoopAnchor,
            context,
        ));
        node_pixels.push(vec![anchor]);
    }

    ComponentGraph {
        nodes,
        node_pixels,
        pixel_to_node,
    }
}

/// Baut die Welt-Polyline eines Segments aus den Start-/End-Knoten und Kettenpixeln.
fn build_segment_polyline(
    start_node: &SkeletonGraphNode,
    end_node: &SkeletonGraphNode,
    chain_pixels: &[Pixel],
    original_mask: &[bool],
    context: SkeletonBuildContext,
) -> Vec<Vec2> {
    let mut polyline = Vec::with_capacity(chain_pixels.len() + 2);
    polyline.push(start_node.world_position);

    if !chain_pixels.is_empty() {
        let refined =
            refine_medial_axis(chain_pixels, original_mask, context.width, context.height);
        polyline.extend(refined_pixels_to_world(&refined, context));
    }

    polyline.push(end_node.world_position);
    polyline
}

/// Verfolgt ein einzelnes Segment ab einem Graph-Knoten durch Grad-2-Kettenpixel.
fn trace_segment(
    start_node: usize,
    start_pixel: Pixel,
    first_pixel: Pixel,
    pixel_set: &HashSet<Pixel>,
    pixel_to_node: &HashMap<Pixel, usize>,
    visited_edges: &mut HashSet<(Pixel, Pixel)>,
) -> Option<(usize, Vec<Pixel>)> {
    let mut prev = start_pixel;
    let mut current = first_pixel;
    let mut chain_pixels = Vec::new();

    loop {
        visited_edges.insert(normalized_edge(prev, current));

        if let Some(&node_id) = pixel_to_node.get(&current) {
            return Some((node_id, chain_pixels));
        }

        chain_pixels.push(current);

        let mut next_candidates: Vec<Pixel> = skeleton_neighbors(current, pixel_set)
            .into_iter()
            .filter(|&candidate| candidate != prev)
            .collect();

        if next_candidates.is_empty() {
            return None;
        }

        next_candidates.sort_unstable();
        let fallback = next_candidates[0];
        let next = next_candidates
            .iter()
            .copied()
            .find(|&candidate| !visited_edges.contains(&normalized_edge(current, candidate)))
            .unwrap_or(fallback);

        prev = current;
        current = next;

        if pixel_to_node.get(&prev) == Some(&start_node) && current == start_pixel {
            return Some((start_node, chain_pixels));
        }
    }
}

/// Extrahiert fuer eine einzelne Komponente deren Teilnetz.
fn extract_component_network(
    component: &[Pixel],
    original_mask: &[bool],
    context: SkeletonBuildContext,
    start_hint: Option<Pixel>,
) -> SkeletonNetwork {
    let pixel_set: HashSet<Pixel> = component.iter().copied().collect();
    let degrees: HashMap<Pixel, usize> = component
        .iter()
        .copied()
        .map(|pixel| (pixel, skeleton_neighbors(pixel, &pixel_set).len()))
        .collect();

    let graph = build_component_graph(component, &degrees, context, start_hint);

    let mut segments = Vec::new();
    let mut visited_edges: HashSet<(Pixel, Pixel)> = HashSet::new();

    for (node_id, cluster_pixels) in graph.node_pixels.iter().enumerate() {
        let mut frontier_map: HashMap<Pixel, Pixel> = HashMap::new();
        for &node_pixel in cluster_pixels {
            for neighbor in skeleton_neighbors(node_pixel, &pixel_set) {
                if graph.pixel_to_node.get(&neighbor) == Some(&node_id) {
                    continue;
                }
                frontier_map.entry(neighbor).or_insert(node_pixel);
            }
        }

        let mut frontiers: Vec<(Pixel, Pixel)> = frontier_map.into_iter().collect();
        frontiers.sort_unstable_by_key(|&(neighbor, start_pixel)| (neighbor, start_pixel));

        for (neighbor, start_pixel) in frontiers {
            let edge = normalized_edge(start_pixel, neighbor);
            if visited_edges.contains(&edge) {
                continue;
            }

            let Some((end_node, chain_pixels)) = trace_segment(
                node_id,
                start_pixel,
                neighbor,
                &pixel_set,
                &graph.pixel_to_node,
                &mut visited_edges,
            ) else {
                continue;
            };

            let polyline = build_segment_polyline(
                &graph.nodes[node_id],
                &graph.nodes[end_node],
                &chain_pixels,
                original_mask,
                context,
            );
            if polyline.len() >= 2 {
                segments.push(SkeletonGraphSegment {
                    start_node: node_id,
                    end_node,
                    polyline,
                });
            }
        }
    }

    SkeletonNetwork {
        nodes: graph.nodes,
        segments,
    }
}

// ---------------------------------------------------------------------------
// Skelett-Pfad ordnen (Durchmesser-BFS)
// ---------------------------------------------------------------------------

/// Ordnet eine Menge von Skelett-Pixeln in eine lineare Sequenz.
///
/// Algorithmus: Zweifache BFS (Durchmesser-Methode).
/// 1. BFS vom Startpunkt (Hint-Pixel oder beliebig) → findet Endpunkt A.
/// 2. BFS von A → findet Endpunkt B und rekonstruiert den laengsten Pfad A→B.
///
/// Ist `hint` angegeben, wird als erster Startpunkt der Pixel aus `pixels`
/// gewaehlt der dem Hint am naechsten liegt. Dadurch laeuft der Pfad von
/// der Lasso-Startseite aus, nicht vom geometrischen Durchmesser-Endpunkt.
///
/// Bei Verzweigungen wird automatisch der laengste Teilpfad gewaehlt,
/// da der Graphdurchmesser immer die zwei weitesten Endpunkte verbindet.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn order_skeleton_pixels(
    pixels: &[(usize, usize)],
    hint: Option<(usize, usize)>,
) -> Vec<(usize, usize)> {
    if pixels.is_empty() {
        return Vec::new();
    }
    if pixels.len() == 1 {
        return vec![pixels[0]];
    }

    let pixel_set: std::collections::HashSet<(usize, usize)> = pixels.iter().copied().collect();

    // Startpunkt: Pixel am naechsten zum Hint (oder erstes Element als Fallback)
    let initial_start = if let Some((hx, hy)) = hint {
        pixels
            .iter()
            .copied()
            .min_by_key(|&(px, py)| {
                let dx = px as i64 - hx as i64;
                let dy = py as i64 - hy as i64;
                dx * dx + dy * dy
            })
            .unwrap_or(pixels[0])
    } else {
        pixels[0]
    };

    // Rueckgabetyp-Alias fuer die BFS-Hilfsclosure (farthest_node + parent_map)
    type BfsResult = (
        (usize, usize),
        HashMap<(usize, usize), Option<(usize, usize)>>,
    );

    // BFS von einem Startknoten: gibt (farthest_node, parent_map) zurueck.
    // Die parent_map erlaubt die Pfad-Rekonstruktion vom farthest_node
    // zurueck zum Startknoten.
    let bfs_from = |start: (usize, usize)| -> BfsResult {
        let mut queue = VecDeque::new();
        let mut parent: HashMap<(usize, usize), Option<(usize, usize)>> = HashMap::new();
        queue.push_back(start);
        parent.insert(start, None);
        let mut farthest = start;

        while let Some(current) = queue.pop_front() {
            farthest = current;
            let (cx, cy) = current;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = cx as i32 + dx;
                    let ny = cy as i32 + dy;
                    if nx < 0 || ny < 0 {
                        continue;
                    }
                    let nbr = (nx as usize, ny as usize);
                    if pixel_set.contains(&nbr) && !parent.contains_key(&nbr) {
                        parent.insert(nbr, Some(current));
                        queue.push_back(nbr);
                    }
                }
            }
        }
        (farthest, parent)
    };

    // Schritt 1: BFS vom Startpunkt → Endpunkt A (einer der Durchmesser-Enden)
    let (far_a, _) = bfs_from(initial_start);

    // Schritt 2: BFS von A → Endpunkt B + Parent-Map fuer Pfad-Rekonstruktion
    let (far_b, parent_map) = bfs_from(far_a);

    // Pfad von B zurueck zu A rekonstruieren
    let mut path = Vec::new();
    let mut current = far_b;
    loop {
        path.push(current);
        match parent_map[&current] {
            Some(p) => current = p,
            None => break, // Startknoten A erreicht
        }
    }

    // Pfad laeuft B→A; umkehren fuer A→B
    path.reverse();
    path
}

// ---------------------------------------------------------------------------
// Medial-Axis-Korrektur
// ---------------------------------------------------------------------------

/// Sucht den Abstand zum naechsten Rand-Pixel in einer Richtung (nx, ny).
///
/// Schrittweise Abtastung entlang (nx, ny) ab (x, y). Gibt die Distanz (in
/// Pixeln − 0.5) zurueck, an der erstmals ein `false`-Pixel oder der
/// Bildrand erreicht wird.
fn find_boundary_distance(
    x: usize,
    y: usize,
    nx: f32,
    ny: f32,
    mask: &[bool],
    width: usize,
    height: usize,
) -> f32 {
    for step in 1..=30i32 {
        let ix = (x as f32 + nx * step as f32).round() as i32;
        let iy = (y as f32 + ny * step as f32).round() as i32;
        if ix < 0 || iy < 0 || ix >= width as i32 || iy >= height as i32 {
            return step as f32 - 0.5;
        }
        if !mask[iy as usize * width + ix as usize] {
            return step as f32 - 0.5;
        }
    }
    30.0
}

/// Korrigiert geordnete Skelett-Pixel auf die geometrische Mittelachse.
///
/// Fuer jeden Skelett-Pixel wird die lokale Tangente aus Vorgaenger und
/// Nachfolger berechnet. Senkrecht dazu wird auf beiden Seiten der naechste
/// Rand-Pixel in `original_mask` gesucht. Der korrigierte Punkt liegt auf
/// dem geometrischen Mittelpunkt zwischen beiden Raendern.
pub(crate) fn refine_medial_axis(
    ordered: &[(usize, usize)],
    original_mask: &[bool],
    width: usize,
    height: usize,
) -> Vec<(f32, f32)> {
    let n = ordered.len();
    ordered
        .iter()
        .enumerate()
        .map(|(i, &(x, y))| {
            let (prev_x, prev_y) = if i > 0 {
                ordered[i - 1]
            } else if i + 1 < n {
                ordered[i + 1]
            } else {
                (x, y)
            };
            let (next_x, next_y) = if i + 1 < n {
                ordered[i + 1]
            } else if i > 0 {
                ordered[i - 1]
            } else {
                (x, y)
            };

            let dx = next_x as f32 - prev_x as f32;
            let dy = next_y as f32 - prev_y as f32;
            let len = (dx * dx + dy * dy).sqrt();

            if len < 0.001 {
                return (x as f32, y as f32);
            }

            // Normierte Tangente; Normale = 90°-Rotation
            let (tx, ty) = (dx / len, dy / len);
            let (nx_f, ny_f) = (-ty, tx);

            // Abstand zum Rand in beiden Normalenrichtungen
            let d_pos = find_boundary_distance(x, y, nx_f, ny_f, original_mask, width, height);
            let d_neg = find_boundary_distance(x, y, -nx_f, -ny_f, original_mask, width, height);

            // Mittelachsen-Offset: positiv = in Richtung +Normale
            let offset = (d_pos - d_neg) / 2.0;
            (x as f32 + nx_f * offset, y as f32 + ny_f * offset)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Pixel → Weltkoordinaten
// ---------------------------------------------------------------------------

/// Konvertiert korrigierte Sub-Pixel-Positionen in Weltkoordinaten.
///
/// Wird nach `refine_medial_axis` verwendet, wo Pixel-Positionen nicht
/// ganzzahlig sein koennen.
fn refined_pixels_to_world(refined: &[(f32, f32)], context: SkeletonBuildContext) -> Vec<Vec2> {
    refined
        .iter()
        .map(|&(px, py)| context.pixel_to_world(px, py))
        .collect()
}

// ---------------------------------------------------------------------------
// Haupt-Pipeline
// ---------------------------------------------------------------------------

/// Fuehrt die komplette Netz-Pipeline aus:
/// Bool-Maske → Zhang-Suen → Komponenten → Graph-Knoten/Segmente → Weltkoords.
///
/// Gibt ein zusammenhaengendes Teilnetz mit Kreuzungen, offenen Enden und Segmenten zurueck.
/// Komponenten unterhalb der Mindestgroesse werden verworfen.
///
/// - `noise_filter`: Wenn `true`, wird vor dem Thinning morphologisches
///   Opening (Erosion+Dilation) und Closing (Dilation+Erosion) angewendet
///   um Einzelpixel-Rauschen zu entfernen und kleine Luecken zu schliessen.
/// - `start_hint`: Optionaler Pixel-Punkt in der Naehe des Lasso-Startpunkts.
///   Steuert bei geschlossenen Schleifen den kuenstlichen Start-/Ankerpunkt.
pub(crate) fn extract_network_from_mask(
    mask: &mut Vec<bool>,
    width: u32,
    height: u32,
    noise_filter: bool,
    map_size: f32,
    start_hint: Option<(usize, usize)>,
) -> SkeletonNetwork {
    let w = width as usize;
    let h = height as usize;
    let context = SkeletonBuildContext {
        width: w,
        height: h,
        map_size,
        img_width: width,
        img_height: height,
    };

    // Optional: Rauschfilter — Opening entfernt isolierte Pixel,
    // Closing schliesst kleine Luecken
    if noise_filter {
        let opened = morphological_open(mask, w, h);
        let closed = morphological_close(&opened, w, h);
        *mask = closed;
    }

    // Original-Maske vor Zhang-Suen sichern (fuer Medial-Axis-Korrektur)
    let original_mask = mask.clone();

    // Zhang-Suen: Maske auf 1-Pixel-breites Skelett reduzieren
    zhang_suen_thinning(mask, w, h);

    // Zusammenhaengende Skelett-Gruppen extrahieren
    let components = find_connected_components(mask, w, h);

    let mut kept_components = 0usize;
    let mut network = SkeletonNetwork::default();

    for component in components
        .into_iter()
        .filter(|comp| comp.len() >= MIN_COMPONENT_PIXELS)
    {
        kept_components += 1;
        let component_network =
            extract_component_network(&component, &original_mask, context, start_hint);

        let node_offset = network.nodes.len();
        network.nodes.extend(component_network.nodes);
        network
            .segments
            .extend(component_network.segments.into_iter().map(|mut segment| {
                segment.start_node += node_offset;
                segment.end_node += node_offset;
                segment
            }));
    }

    log::info!(
        "Skelett: {} Komponenten → {} Knoten ({} Kreuzungen, {} offene Enden) und {} Segmente",
        kept_components,
        network.nodes.len(),
        network.junction_count(),
        network.open_end_count(),
        network.segments.len()
    );

    network
}

/// Legacy-Helfer fuer lineare Pfad-Konsumenten und bestehende Tests.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn extract_paths_from_mask(
    mask: &mut Vec<bool>,
    width: u32,
    height: u32,
    noise_filter: bool,
    map_size: f32,
    start_hint: Option<(usize, usize)>,
) -> Vec<Vec<Vec2>> {
    extract_network_from_mask(mask, width, height, noise_filter, map_size, start_hint)
        .segments
        .into_iter()
        .map(|segment| segment.polyline)
        .collect()
}

// ---------------------------------------------------------------------------
// Unit-Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_mask(width: usize, height: usize) -> Vec<bool> {
        vec![false; width * height]
    }

    fn set_pixel(mask: &mut [bool], x: usize, y: usize, width: usize) {
        mask[y * width + x] = true;
    }

    /// Zwei getrennte L-foermige Gruppen werden als separate Komponenten erkannt.
    #[test]
    fn connected_components_zwei_getrennte_gruppen() {
        let width = 10usize;
        let height = 5usize;
        let mut mask = empty_mask(width, height);

        // Gruppe 1: L-Form bei (0,0)
        set_pixel(&mut mask, 0, 0, width);
        set_pixel(&mut mask, 0, 1, width);
        set_pixel(&mut mask, 0, 2, width);
        set_pixel(&mut mask, 1, 2, width);

        // Gruppe 2: L-Form bei (7,0) — weit genug entfernt fuer keine 8-Nachbarschaft
        set_pixel(&mut mask, 7, 0, width);
        set_pixel(&mut mask, 7, 1, width);
        set_pixel(&mut mask, 7, 2, width);
        set_pixel(&mut mask, 8, 2, width);

        let components = find_connected_components(&mask, width, height);
        assert_eq!(components.len(), 2, "Zwei Gruppen erwartet");
        assert_eq!(components[0].len(), 4, "Gruppe 1: 4 Pixel");
        assert_eq!(components[1].len(), 4, "Gruppe 2: 4 Pixel");
    }

    /// Leere Maske ergibt keine Komponenten.
    #[test]
    fn connected_components_leere_maske() {
        let mask = empty_mask(5, 5);
        let components = find_connected_components(&mask, 5, 5);
        assert!(components.is_empty(), "Keine Komponenten in leerer Maske");
    }

    /// Einzelner Pixel ergibt eine Komponente mit einem Element.
    #[test]
    fn connected_components_einzelner_pixel() {
        let mut mask = empty_mask(5, 5);
        set_pixel(&mut mask, 2, 2, 5);
        let components = find_connected_components(&mask, 5, 5);
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].len(), 1);
    }

    /// Linearer 5-Pixel-Pfad wird korrekt geordnet (alle Pixel enthalten, richtige Endpunkte).
    #[test]
    fn order_linear_pfad_fuenf_pixel() {
        let pixels = vec![(0, 2), (1, 2), (2, 2), (3, 2), (4, 2)];
        let ordered = order_skeleton_pixels(&pixels, None);
        assert_eq!(ordered.len(), 5, "Alle 5 Pixel muessen enthalten sein");

        // Endpunkte muessen (0,2) und (4,2) sein (Reihenfolge egal)
        let ends: std::collections::HashSet<(usize, usize)> =
            [ordered[0], ordered[4]].iter().copied().collect();
        assert!(
            ends.contains(&(0, 2)),
            "Endpunkt (0,2) muss im Ergebnis sein"
        );
        assert!(
            ends.contains(&(4, 2)),
            "Endpunkt (4,2) muss im Ergebnis sein"
        );
    }

    /// Bei einer Y-Form (Stamm + kurzer Ast) wird der laengste Teilpfad gewaehlt.
    ///
    /// Geometrie:
    /// - Vertikaler Stamm: (2,0) bis (2,5) — 6 Pixel
    /// - Kurzer Ast am Knoten (2,3): Pixel (3,3) — 1 Pixel
    ///
    /// Erwartung: Ergebnis = 6 Pixel (Stamm), Ast (3,3) nicht im Hauptpfad.
    #[test]
    fn order_verzweigung_laengster_pfad() {
        // Vertikaler Stamm: 6 Pixel
        let mut pixels = vec![(2, 0), (2, 1), (2, 2), (2, 3), (2, 4), (2, 5)];
        // Kurzer Ast — per 8-Connectivity mit (2,2), (2,3) und (2,4) verbunden
        pixels.push((3, 3));

        let ordered = order_skeleton_pixels(&pixels, None);
        assert_eq!(
            ordered.len(),
            6,
            "Nur der Stamm (6 Pixel) soll im Pfad sein; Ast (3,3) wird ausgeschlossen"
        );

        // Endpunkte muessen (2,0) und (2,5) sein
        let ends: std::collections::HashSet<(usize, usize)> =
            [ordered[0], ordered[5]].iter().copied().collect();
        assert!(
            ends.contains(&(2, 0)),
            "Stamm-Endpunkt (2,0) muss Pfad-Endpunkt sein"
        );
        assert!(
            ends.contains(&(2, 5)),
            "Stamm-Endpunkt (2,5) muss Pfad-Endpunkt sein"
        );
    }

    /// Eine 3-Pixel-breite horizontale Linie ergibt nach Thinning einen einzelnen Pfad.
    #[test]
    fn extract_paths_horizontale_linie_3px_breit() {
        let width = 12u32;
        let height = 7u32;
        let w = width as usize;

        // 3 Pixel breites Band: y=2,3,4; innere Pixel x=1..=10 (Rand bleibt false)
        let mut mask = vec![false; (width * height) as usize];
        for y in 2usize..=4 {
            for x in 1usize..=10 {
                mask[y * w + x] = true;
            }
        }

        let paths = extract_paths_from_mask(&mut mask, width, height, false, 1000.0, None);

        assert_eq!(paths.len(), 1, "Genau ein Pfad nach Thinning erwartet");
        assert!(
            paths[0].len() >= 5,
            "Pfad muss mindestens 5 Punkte haben, hat: {}",
            paths[0].len()
        );
    }

    /// Ein T-Knoten wird als ein Junction-Cluster mit drei Segmenten erkannt.
    #[test]
    fn extract_network_t_knoten_liefert_junction_und_segmente() {
        let width = 7u32;
        let height = 7u32;
        let w = width as usize;
        let mut mask = vec![false; (width * height) as usize];

        for y in 1usize..=5 {
            set_pixel(&mut mask, 3, y, w);
        }
        for x in 1usize..=5 {
            set_pixel(&mut mask, x, 3, w);
        }

        let network = extract_network_from_mask(&mut mask, width, height, false, 1000.0, None);

        assert_eq!(network.junction_count(), 1);
        assert_eq!(network.open_end_count(), 4);
        assert_eq!(network.segments.len(), 4);
    }

    /// Zwei benachbarte Branch-Pixel werden zu genau einer Junction zusammengefasst.
    #[test]
    fn adjacent_branch_pixels_are_clustered_into_one_junction() {
        let width = 8usize;
        let height = 6usize;
        let mut mask = vec![false; width * height];
        let component = vec![
            (0, 2),
            (1, 2),
            (2, 2),
            (3, 2),
            (4, 2),
            (5, 2),
            (2, 1),
            (2, 0),
            (3, 3),
            (3, 4),
        ];

        for &(x, y) in &component {
            set_pixel(&mut mask, x, y, width);
        }

        let context = SkeletonBuildContext {
            width,
            height,
            map_size: 1000.0,
            img_width: width as u32,
            img_height: height as u32,
        };

        let network = extract_component_network(&component, &mask, context, None);

        assert_eq!(network.junction_count(), 1);
        assert_eq!(network.open_end_count(), 4);
        assert_eq!(network.segments.len(), 4);
    }
}
