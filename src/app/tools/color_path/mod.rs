//! Farb-Pfad-Tool: erkennt Wege anhand der Farbe im Hintergrundbild.

mod config_ui;
mod lifecycle;
mod pipeline;
mod preview;
pub(crate) mod sampling;
pub(crate) mod skeleton;
mod state;

pub use state::ColorPathTool;

/// Fuehrt die Kernpipeline des ColorPathTool fuer Benchmarks und Analysen aus.
///
/// Die Funktion kapselt Flood-Fill und Netzextraktion, ohne interne
/// Skelett-Typen nach aussen zu exponieren. Rueckgabe:
/// `(node_count, segment_count, junction_count, open_end_count)`.
pub fn compute_color_path_network_stats(
    image: &image::DynamicImage,
    palette: &[[u8; 3]],
    tolerance: f32,
    start_pixel: (u32, u32),
    noise_filter: bool,
    map_size: f32,
) -> (usize, usize, usize, usize) {
    let (mask, width, height) =
        sampling::flood_fill_color_mask(image, palette, tolerance, start_pixel);
    let prepared_mask =
        sampling::prepare_mask_for_skeleton(&mask, width as usize, height as usize, noise_filter);
    let start_hint = Some((start_pixel.0 as usize, start_pixel.1 as usize));
    let network =
        skeleton::extract_network_from_mask(&prepared_mask, width, height, map_size, start_hint);

    (
        network.nodes.len(),
        network.segments.len(),
        network.junction_count(),
        network.open_end_count(),
    )
}
