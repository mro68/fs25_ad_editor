//! Fingerabdruck-Mechanismus fuer Render-Buffer-Skip-Detection.
//!
//! Jeder Sub-Renderer speichert nach einem erfolgreichen Render-Pass den `RenderFingerprint`
//! seiner Eingabe-Daten. Vor dem naechsten Rebuild wird ein neuer Fingerabdruck berechnet
//! und mit dem gespeicherten verglichen. Bei Uebereinstimmung kann der CPU-seitige
//! Buffer-Aufbau und der GPU-Upload uebersprungen werden — der Draw-Call laeuft weiterhin.
//!
//! # Pointer-Vergleiche
//!
//! Fuer Arc-Inhalte (RoadMap, EditorOptions, IndexSets) wird die stabile Adresse der
//! Arc-internen Daten (`Arc::as_ptr()` bzw. `&*arc as *const T`) als usize verglichen.
//! Da diese Typen ausschliesslich als neues Arc ersetzt werden (Copy-on-Write-Semantik),
//! zeigt ein veraenderter Pointer zuverlaessig auf geaenderte Inhalte.
//!
//! # Float-Vergleiche
//!
//! Kamera- und Viewport-Floats werden als IEEE-754-Bit-Muster (u32) verglichen,
//! um NaN-Gleichheitsprobleme zu vermeiden und exakte Frame-zu-Frame-Aenderungen zu erkennen.

use crate::shared::EditorOptions;
use crate::RoadMap;
use indexmap::IndexSet;

use super::types::RenderContext;

/// Fingerabdruck der Render-Inputs eines Sub-Renderers.
///
/// Wird am Ende eines erfolgreichen Render-Passes gespeichert. Beim naechsten Aufruf
/// wird ein neuer Fingerabdruck berechnet und mit dem gespeicherten verglichen.
/// Bei Uebereinstimmung koennen Buffer-Aufbau und GPU-Upload uebersprungen werden.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct RenderFingerprint {
    /// Pointer-Adresse der RoadMap-Daten (0 = keine Map).
    pub road_map_ptr: usize,
    /// Pointer-Adresse der EditorOptions-Daten.
    pub options_ptr: usize,
    /// Pointer-Adresse der HiddenNodeIds-Daten.
    pub hidden_ptr: usize,
    /// Pointer-Adresse der DimmedNodeIds-Daten (0 = nicht verwendet).
    pub dimmed_ptr: usize,
    /// Pointer-Adresse der SelectedNodeIds-Daten (0 = nicht verwendet).
    pub selected_ptr: usize,
    /// Kamera-Position X als IEEE-754-Bit-Muster.
    pub camera_x: u32,
    /// Kamera-Position Y als IEEE-754-Bit-Muster.
    pub camera_y: u32,
    /// Kamera-Zoom als IEEE-754-Bit-Muster.
    pub camera_zoom: u32,
    /// Viewport-Breite als IEEE-754-Bit-Muster.
    pub viewport_w: u32,
    /// Viewport-Hoehe als IEEE-754-Bit-Muster.
    pub viewport_h: u32,
    /// Render-Qualitaetsstufe als Diskriminant (0 = nicht relevant).
    pub quality: u8,
}

impl RenderFingerprint {
    /// Erstellt einen Basis-Fingerabdruck aus dem gemeinsamen Render-Kontext und der RoadMap.
    ///
    /// Renderer-spezifische Felder (`dimmed_ptr`, `selected_ptr`, `quality`) sind auf
    /// Null initialisiert und muessen vom Aufrufer bei Bedarf manuell gesetzt werden.
    pub fn from_context(ctx: &RenderContext<'_>, road_map: &RoadMap) -> Self {
        Self {
            road_map_ptr: road_map as *const RoadMap as usize,
            options_ptr: ctx.options as *const EditorOptions as usize,
            hidden_ptr: ctx.hidden_node_ids as *const IndexSet<u64> as usize,
            dimmed_ptr: 0,
            selected_ptr: 0,
            camera_x: ctx.camera.position.x.to_bits(),
            camera_y: ctx.camera.position.y.to_bits(),
            camera_zoom: ctx.camera.zoom.to_bits(),
            viewport_w: ctx.viewport_size[0].to_bits(),
            viewport_h: ctx.viewport_size[1].to_bits(),
            quality: 0,
        }
    }
}
