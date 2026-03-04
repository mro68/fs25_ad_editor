//! Benchmark fuer Route-Tool-Preview-Hotpaths.
//!
//! Misst die Kernberechnungen der Preview-Erzeugung in den aufwaendigeren Tools:
//! - Bypass: `compute_bypass_positions`
//! - ConstraintRoute: `solve_route`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fs25_auto_drive_editor::app::tools::bypass::compute_bypass_positions;
use fs25_auto_drive_editor::app::tools::constraint_route::{solve_route, ConstraintRouteInput};
use glam::Vec2;
use std::hint::black_box;

fn make_chain(count: usize, step: f32) -> Vec<Vec2> {
    (0..count)
        .map(|i| {
            let x = i as f32 * step;
            let y = (i as f32 * 0.3).sin() * 7.5;
            Vec2::new(x, y)
        })
        .collect()
}

fn bench_bypass_preview(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_preview_bypass");

    for &count in &[10usize, 50, 200] {
        let chain = make_chain(count, 6.0);

        group.bench_with_input(BenchmarkId::new("compute_positions", count), &chain, |b, pts| {
            b.iter(|| {
                let result = compute_bypass_positions(black_box(pts), 8.0, 6.0);
                black_box(result)
            })
        });
    }

    group.finish();
}

fn bench_constraint_preview(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_preview_constraint_route");

    for &controls in &[0usize, 3, 8] {
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(120.0, 30.0);
        let control_nodes: Vec<Vec2> = (0..controls)
            .map(|i| Vec2::new(15.0 + i as f32 * 12.0, (i as f32 * 0.8).sin() * 18.0))
            .collect();

        let input = ConstraintRouteInput {
            start,
            end,
            control_nodes,
            max_segment_length_m: 6.0,
            max_direction_change_deg: 45.0,
            start_neighbor_directions: vec![Vec2::new(1.0, 0.0), Vec2::new(0.7, 0.3)],
            end_neighbor_directions: vec![Vec2::new(-1.0, 0.0), Vec2::new(-0.7, -0.3)],
            min_distance: 1.0,
        };

        group.bench_with_input(
            BenchmarkId::new("solve_route", controls),
            &input,
            |b, route_input| {
                b.iter(|| {
                    let result = solve_route(black_box(route_input));
                    black_box(result.positions.len())
                })
            },
        );
    }

    group.finish();
}

criterion_group!(tool_preview_hotpath_benches, bench_bypass_preview, bench_constraint_preview);
criterion_main!(tool_preview_hotpath_benches);
