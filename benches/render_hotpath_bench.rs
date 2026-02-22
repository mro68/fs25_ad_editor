//! Benchmark für Render-Hotpath-Allokationen.
//!
//! Misst die Datenaufbereitungskosten pro Frame in den Renderern:
//! - NodeRenderer: HashSet-Erstellung aus selected_node_ids (Slice → HashSet)
//! - MarkerRenderer: Vec-collect für MarkerInstances
//! - RenderScene: selected_node_ids.iter().copied().collect()

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fs25_auto_drive_editor::{MapMarker, MapNode, NodeFlag, RoadMap};
use glam::Vec2;
use std::collections::HashSet;
use std::hint::black_box;

fn build_road_map_with_markers(node_count: usize, marker_count: usize) -> RoadMap {
    let mut road_map = RoadMap::new(3);

    for i in 0..node_count {
        let id = (i as u64) + 1;
        let x = (i % 1000) as f32 + (i as f32 * 0.0017).fract();
        let y = (i / 1000) as f32 + (i as f32 * 0.0031).fract();
        road_map
            .nodes
            .insert(id, MapNode::new(id, Vec2::new(x, y), NodeFlag::Regular));
    }

    for i in 0..marker_count.min(node_count) {
        let id = (i as u64) + 1;
        road_map.map_markers.push(MapMarker {
            id,
            name: format!("Marker {}", i),
            group: "default".to_string(),
            marker_index: i as u32,
            is_debug: false,
        });
    }

    road_map.rebuild_spatial_index();
    road_map
}

/// Misst: HashSet<u64> aus einem Slice erstellen (= NodeRenderer pro Frame)
fn bench_hashset_from_selected_ids(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_renderer_hashset");

    for &selected_count in &[0usize, 10, 100, 1000, 10_000] {
        let selected_ids: Vec<u64> = (1..=selected_count as u64).collect();

        group.bench_with_input(
            BenchmarkId::new("selected_to_hashset", selected_count),
            &selected_ids,
            |b, ids| {
                b.iter(|| {
                    let set: HashSet<u64> = black_box(ids).iter().copied().collect();
                    black_box(set.len())
                })
            },
        );
    }

    group.finish();
}

/// Misst: Vec<u64> collect aus HashSet (= render_scene::build pro Frame)
fn bench_render_scene_collect(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_scene_collect");

    for &selected_count in &[0usize, 10, 100, 1000, 10_000] {
        let selected_set: HashSet<u64> = (1..=selected_count as u64).collect();

        group.bench_with_input(
            BenchmarkId::new("hashset_to_vec", selected_count),
            &selected_set,
            |b, set| {
                b.iter(|| {
                    let vec: Vec<u64> = black_box(set).iter().copied().collect();
                    black_box(vec.len())
                })
            },
        );
    }

    group.finish();
}

/// Misst: Marker-Instance-Aufbau (filter_map + collect, simuliert)
fn bench_marker_instance_collect(c: &mut Criterion) {
    let mut group = c.benchmark_group("marker_renderer_collect");

    for &(node_count, marker_count) in &[(1000, 10), (10_000, 50), (100_000, 200)] {
        let road_map = build_road_map_with_markers(node_count, marker_count);

        group.bench_with_input(
            BenchmarkId::new(
                "marker_filter_collect",
                format!("{}n_{}m", node_count, marker_count),
            ),
            &road_map,
            |b, rm| {
                b.iter(|| {
                    // Simuliert MarkerRenderer.render() Datenaufbereitung
                    let instances: Vec<(f32, f32)> = rm
                        .map_markers
                        .iter()
                        .filter_map(|marker| {
                            let node = rm.nodes.get(&marker.id)?;
                            Some((node.position.x, node.position.y))
                        })
                        .collect();
                    black_box(instances.len())
                })
            },
        );
    }

    group.finish();
}

/// Misst: Node-Instance-Aufbau mit contains-Check auf HashSet (simuliert NodeRenderer)
fn bench_node_instance_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_renderer_instance_build");

    for &node_count in &[10_000usize, 100_000] {
        let road_map = build_road_map_with_markers(node_count, 0);
        let visible_ids: Vec<u64> = (1..=(node_count.min(5000) as u64)).collect();
        let selected: HashSet<u64> = (1..=100).collect();

        group.bench_with_input(
            BenchmarkId::new("build_instances", node_count),
            &(&road_map, &visible_ids, &selected),
            |b, &(rm, vis, sel)| {
                b.iter(|| {
                    let mut instances: Vec<(f32, f32, bool)> = Vec::new();
                    for &node_id in vis {
                        if let Some(node) = rm.nodes.get(&node_id) {
                            let is_selected = sel.contains(&node.id);
                            instances.push((node.position.x, node.position.y, is_selected));
                        }
                    }
                    black_box(instances.len())
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    render_hotpath_benches,
    bench_hashset_from_selected_ids,
    bench_render_scene_collect,
    bench_marker_instance_collect,
    bench_node_instance_build,
);
criterion_main!(render_hotpath_benches);
