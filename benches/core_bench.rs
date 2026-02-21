use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fs25_auto_drive_editor::{parse_autodrive_config, MapNode, NodeFlag, RoadMap};
use glam::Vec2;
use std::hint::black_box;

fn bench_xml_parsing(c: &mut Criterion) {
    let xml_content = include_str!("../tests/fixtures/simple_config.xml");

    c.bench_function("xml_parse_simple_config", |b| {
        b.iter(|| {
            let map = parse_autodrive_config(black_box(xml_content)).expect("XML parse failed");
            black_box(map.node_count())
        })
    });
}

fn build_synthetic_road_map(node_count: usize) -> RoadMap {
    let mut road_map = RoadMap::new(3);

    for index in 0..node_count {
        let id = (index as u64) + 1;
        let column = (index % 1000) as f32;
        let row = (index / 1000) as f32;
        let x = column + row * 0.001;
        let y = row + column * 0.001;
        road_map
            .nodes
            .insert(id, MapNode::new(id, Vec2::new(x, y), NodeFlag::Regular));
    }

    road_map.rebuild_spatial_index();

    road_map
}

fn build_query_points(count: usize) -> Vec<Vec2> {
    (0..count)
        .map(|i| {
            let x = (i % 1000) as f32 + 0.37;
            let y = ((i * 7) % 1000) as f32 + 0.63;
            Vec2::new(x, y)
        })
        .collect()
}

fn bench_spatial_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_queries");

    for &node_count in &[10_000usize, 100_000usize] {
        let road_map = build_synthetic_road_map(node_count);
        let query_points = build_query_points(1024);

        group.bench_with_input(
            BenchmarkId::new("nearest_batch", node_count),
            &road_map,
            |b, map| {
                b.iter(|| {
                    let mut hits = 0usize;
                    for point in &query_points {
                        if map.nearest_node(black_box(*point)).is_some() {
                            hits += 1;
                        }
                    }
                    black_box(hits)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rect_query", node_count),
            &road_map,
            |b, map| {
                b.iter(|| {
                    let ids = map.nodes_within_rect(
                        black_box(Vec2::new(250.0, 10.0)),
                        black_box(Vec2::new(750.0, 90.0)),
                    );
                    black_box(ids.len())
                })
            },
        );
    }

    group.finish();
}

criterion_group!(core_benches, bench_xml_parsing, bench_spatial_queries);
criterion_main!(core_benches);
