//! Fingerabdruck-Mechanismus fuer Render-Buffer-Skip-Detection.
//!
//! Jeder Sub-Renderer speichert nach einem erfolgreichen Render-Pass den `RenderFingerprint`
//! seiner Eingabe-Daten. Vor dem naechsten Rebuild wird ein neuer Fingerabdruck berechnet
//! und mit dem gespeicherten verglichen. Bei Uebereinstimmung kann der CPU-seitige
//! Buffer-Aufbau und der GPU-Upload uebersprungen werden — der Draw-Call laeuft weiterhin.
//!
//! # Pointer-Vergleiche
//!
//! Fuer Arc-Inhalte (RenderMap, EditorOptions, IndexSets) wird die stabile Adresse der
//! Arc-internen Daten (`Arc::as_ptr()` bzw. `&*arc as *const T`) als usize verglichen.
//! Da diese Typen ausschliesslich als neues Arc ersetzt werden (Copy-on-Write-Semantik),
//! zeigt ein veraenderter Pointer zuverlaessig auf geaenderte Inhalte.
//!
//! # Float-Vergleiche
//!
//! Kamera- und Viewport-Floats werden als IEEE-754-Bit-Muster (u32) verglichen,
//! um NaN-Gleichheitsprobleme zu vermeiden und exakte Frame-zu-Frame-Aenderungen zu erkennen.

use crate::shared::{EditorOptions, RenderMap};
use indexmap::IndexSet;

use super::types::RenderContext;

/// Fingerabdruck der Render-Inputs eines Sub-Renderers.
///
/// Wird am Ende eines erfolgreichen Render-Passes gespeichert. Beim naechsten Aufruf
/// wird ein neuer Fingerabdruck berechnet und mit dem gespeicherten verglichen.
/// Bei Uebereinstimmung koennen Buffer-Aufbau und GPU-Upload uebersprungen werden.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct RenderFingerprint {
    /// Pointer-Adresse der RenderMap-Daten (0 = keine Map).
    pub render_map_ptr: usize,
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
    /// Erstellt einen Basis-Fingerabdruck aus dem gemeinsamen Render-Kontext und der RenderMap.
    ///
    /// Renderer-spezifische Felder (`dimmed_ptr`, `selected_ptr`, `quality`) sind auf
    /// Null initialisiert und muessen vom Aufrufer bei Bedarf manuell gesetzt werden.
    pub fn from_context(ctx: &RenderContext<'_>, render_map: &RenderMap) -> Self {
        Self {
            render_map_ptr: render_map as *const RenderMap as usize,
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

#[cfg(test)]
mod tests {
    use super::RenderFingerprint;

    /// Hilfsfunktion: Erzeugt einen Fingerabdruck mit definierten Testwerten.
    fn make_fp(ptr: usize, cam_x: f32, cam_y: f32, zoom: f32) -> RenderFingerprint {
        RenderFingerprint {
            render_map_ptr: ptr,
            options_ptr: 0x2000,
            hidden_ptr: 0x3000,
            dimmed_ptr: 0,
            selected_ptr: 0,
            camera_x: cam_x.to_bits(),
            camera_y: cam_y.to_bits(),
            camera_zoom: zoom.to_bits(),
            viewport_w: 800.0f32.to_bits(),
            viewport_h: 600.0f32.to_bits(),
            quality: 0,
        }
    }

    #[test]
    fn gleiche_inputs_ergeben_gleichen_fingerprint() {
        // Zwei identisch konstruierte Fingerabdruecke muessen gleich sein.
        let fp1 = make_fp(0x1000, 10.0, 20.0, 1.5);
        let fp2 = make_fp(0x1000, 10.0, 20.0, 1.5);
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn geaenderter_kamera_zoom_ergibt_anderen_fingerprint() {
        // Zoom-Aenderung muss erkannt werden.
        let fp1 = make_fp(0x1000, 10.0, 20.0, 1.5);
        let fp2 = make_fp(0x1000, 10.0, 20.0, 2.0);
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn geaenderte_kamera_position_ergibt_anderen_fingerprint() {
        // Pan-Bewegung muss zu unterschiedlichem Fingerprint fuehren.
        let fp1 = make_fp(0x1000, 10.0, 20.0, 1.0);
        let fp2 = make_fp(0x1000, 11.0, 20.0, 1.0);
        let fp3 = make_fp(0x1000, 10.0, 21.0, 1.0);
        assert_ne!(fp1, fp2, "Kamera-X geaendert muss ungleich sein");
        assert_ne!(fp1, fp3, "Kamera-Y geaendert muss ungleich sein");
    }

    #[test]
    fn geaenderter_render_map_pointer_ergibt_anderen_fingerprint() {
        // Neuer RenderMap-Snapshot (neuer Arc) muss als Aenderung erkannt werden.
        let fp1 = make_fp(0x1000, 0.0, 0.0, 1.0);
        let fp2 = make_fp(0x2000, 0.0, 0.0, 1.0);
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn default_fingerprint_ist_gleich_sich_selbst() {
        // Default-Fingerprint muss reflexiv gleich sein.
        let fp = RenderFingerprint::default();
        assert_eq!(fp, fp.clone());
    }

    #[test]
    fn ieee754_vergleich_unterscheidet_positive_null_von_normalen_werten() {
        // Sicherstellen, dass 0.0 und 1.0 als Kamera-X unterschiedliche Bits liefern.
        let fp1 = make_fp(0x1000, 0.0, 0.0, 1.0);
        let fp2 = make_fp(0x1000, 1.0, 0.0, 1.0);
        assert_ne!(fp1.camera_x, fp2.camera_x);
    }

    #[test]
    fn quality_feld_wird_im_vergleich_beruecksichtigt() {
        // Render-Qualitaetsstufe muss in den Fingerabdruck einfliessen.
        let mut fp1 = make_fp(0x1000, 0.0, 0.0, 1.0);
        let mut fp2 = fp1.clone();
        fp1.quality = 0;
        fp2.quality = 1;
        assert_ne!(fp1, fp2);
    }
}
