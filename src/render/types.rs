//! Rendering-Typen und Konfiguration.

use crate::shared::EditorOptions;
use crate::Camera2D;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;

pub use crate::shared::RenderQuality;

/// Gemeinsamer Kontext für alle Sub-Renderer.
///
/// Bündelt die GPU-Ressourcen und View-Parameter, die jeder
/// Sub-Renderer bei jedem Frame benötigt.
pub(crate) struct RenderContext<'a> {
    /// wgpu Device für Buffer-Allokation
    pub device: &'a eframe::wgpu::Device,
    /// wgpu Queue für Buffer-Uploads
    pub queue: &'a eframe::wgpu::Queue,
    /// Kamera (Position + Zoom)
    pub camera: &'a Camera2D,
    /// Viewport-Größe in Pixeln [width, height]
    pub viewport_size: [f32; 2],
    /// Editor-Optionen (Farben, Größen, etc.)
    pub options: &'a EditorOptions,
}

/// Vertex für ein Quad (2D-Rechteck)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    /// Position im 2D-Raum
    pub position: [f32; 2],
}

impl Vertex {
    /// Beschreibt das Vertex-Layout für wgpu.
    pub const fn desc() -> eframe::wgpu::VertexBufferLayout<'static> {
        eframe::wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as eframe::wgpu::BufferAddress,
            step_mode: eframe::wgpu::VertexStepMode::Vertex,
            attributes: &[eframe::wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: eframe::wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

/// Vertex für Connection-Geometrie (Linien + Pfeile).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ConnectionVertex {
    /// Position im 2D-Raum
    pub position: [f32; 2],
    /// RGBA-Farbe der Verbindung
    pub color: [f32; 4],
}

impl ConnectionVertex {
    /// Erstellt einen neuen ConnectionVertex.
    pub fn new(position: [f32; 2], color: [f32; 4]) -> Self {
        Self { position, color }
    }

    /// Beschreibt das Vertex-Layout für wgpu.
    pub const fn desc() -> eframe::wgpu::VertexBufferLayout<'static> {
        eframe::wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ConnectionVertex>() as eframe::wgpu::BufferAddress,
            step_mode: eframe::wgpu::VertexStepMode::Vertex,
            attributes: &[
                eframe::wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: eframe::wgpu::VertexFormat::Float32x2,
                },
                eframe::wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as eframe::wgpu::BufferAddress,
                    shader_location: 1,
                    format: eframe::wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Instanz-Daten für einen Node
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct NodeInstance {
    /// Position im 2D-Raum (Weltkoordinaten)
    pub position: [f32; 2],
    /// Basis-Farbe (Mitte des Nodes)
    pub base_color: [f32; 4],
    /// Rand-Farbe (Außenring / Markierung)
    pub rim_color: [f32; 4],
    /// Größe des Nodes in Welteinheiten
    pub size: f32,
    _padding: [f32; 1],
}

impl NodeInstance {
    /// Erstellt eine neue Node-Instanz.
    pub fn new(position: [f32; 2], base_color: [f32; 4], rim_color: [f32; 4], size: f32) -> Self {
        Self {
            position,
            base_color,
            rim_color,
            size,
            _padding: [0.0; 1],
        }
    }

    /// Beschreibt das Instanz-Layout für wgpu (NodeInstance).
    pub const fn desc() -> eframe::wgpu::VertexBufferLayout<'static> {
        eframe::wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<NodeInstance>() as eframe::wgpu::BufferAddress,
            step_mode: eframe::wgpu::VertexStepMode::Instance,
            attributes: &[
                eframe::wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: eframe::wgpu::VertexFormat::Float32x2,
                },
                eframe::wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as eframe::wgpu::BufferAddress,
                    shader_location: 2,
                    format: eframe::wgpu::VertexFormat::Float32x4,
                },
                eframe::wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as eframe::wgpu::BufferAddress,
                    shader_location: 3,
                    format: eframe::wgpu::VertexFormat::Float32x4,
                },
                eframe::wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 10]>() as eframe::wgpu::BufferAddress,
                    shader_location: 4,
                    format: eframe::wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

/// Instanz-Daten für einen Map-Marker (Pin-Symbol)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MarkerInstance {
    /// Position im 2D-Raum (Weltkoordinaten)
    pub position: [f32; 2],
    /// Füllfarbe des Markers
    pub color: [f32; 4],
    /// Outline-Farbe des Markers
    pub outline_color: [f32; 4],
    /// Größe des Markers in Welteinheiten
    pub size: f32,
    _padding: [f32; 1],
}

impl MarkerInstance {
    /// Erstellt eine neue Marker-Instanz.
    pub fn new(position: [f32; 2], color: [f32; 4], outline_color: [f32; 4], size: f32) -> Self {
        Self {
            position,
            color,
            outline_color,
            size,
            _padding: [0.0; 1],
        }
    }

    /// Beschreibt das GPU-Vertex-Buffer-Layout einer `MarkerInstance`.
    pub const fn desc() -> eframe::wgpu::VertexBufferLayout<'static> {
        eframe::wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MarkerInstance>() as eframe::wgpu::BufferAddress,
            step_mode: eframe::wgpu::VertexStepMode::Instance,
            attributes: &[
                eframe::wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: eframe::wgpu::VertexFormat::Float32x2,
                },
                eframe::wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as eframe::wgpu::BufferAddress,
                    shader_location: 2,
                    format: eframe::wgpu::VertexFormat::Float32x4,
                },
                eframe::wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as eframe::wgpu::BufferAddress,
                    shader_location: 3,
                    format: eframe::wgpu::VertexFormat::Float32x4,
                },
                eframe::wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 10]>() as eframe::wgpu::BufferAddress,
                    shader_location: 4,
                    format: eframe::wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

/// Uniform-Buffer für View-Projektion
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Uniforms {
    /// View-Projection-Matrix (4x4)
    pub view_proj: [[f32; 4]; 4],
    /// Anti-Aliasing-Parameter
    pub aa_params: [f32; 4],
}

/// Rendering-Optionen
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Node-Größe in Welteinheiten
    pub node_size: f32,
    /// Ob Sub-Prioritäts-Nodes farblich hervorgehoben werden
    pub highlight_subprio: bool,
    /// Ob Warning-Nodes farblich hervorgehoben werden
    pub highlight_warnings: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            node_size: 5.0,
            highlight_subprio: true,
            highlight_warnings: true,
        }
    }
}

/// Berechnet die View-Projection-Matrix für den 2D-Viewport.
pub(crate) fn build_view_projection(camera: &Camera2D, viewport_size: [f32; 2]) -> Mat4 {
    let view_matrix = camera.view_matrix();
    let aspect = viewport_size[0] / viewport_size[1];
    let zoom_scale = 1.0 / camera.zoom;
    let base_extent = Camera2D::BASE_WORLD_EXTENT;

    let projection = Mat4::orthographic_rh(
        -base_extent * aspect * zoom_scale,
        base_extent * aspect * zoom_scale,
        base_extent * zoom_scale,
        -base_extent * zoom_scale,
        -1.0,
        1.0,
    );

    let view_mat4 = Mat4::from_cols(
        view_matrix.x_axis.extend(0.0),
        view_matrix.y_axis.extend(0.0),
        glam::Vec4::new(0.0, 0.0, 1.0, 0.0),
        view_matrix.z_axis.extend(1.0),
    );

    projection * view_mat4
}
