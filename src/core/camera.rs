//! 2D-Kamera für Pan und Zoom.

use glam::{Mat3, Vec2};

/// 2D-Kamera mit Pan und Zoom
#[derive(Debug, Clone)]
pub struct Camera2D {
    /// Position der Kamera in Welt-Koordinaten
    pub position: Vec2,
    /// Zoom-Level (1.0 = normal, 2.0 = doppelt so groß)
    pub zoom: f32,
}

impl Camera2D {
    /// Sichtbare Welt-Halbbreite bei Zoom 1.0.
    pub const BASE_WORLD_EXTENT: f32 = 2048.0;
    /// Minimaler Zoom-Faktor.
    pub const ZOOM_MIN: f32 = 0.1;
    /// Maximaler Zoom-Faktor.
    pub const ZOOM_MAX: f32 = 100.0;

    /// Erstellt eine neue Kamera
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
        }
    }

    /// Zentriert die Kamera auf einen Punkt
    pub fn look_at(&mut self, target: Vec2) {
        self.position = target;
    }

    /// Verschiebt die Kamera (Pan)
    pub fn pan(&mut self, delta: Vec2) {
        self.position += delta;
    }

    /// Ändert den Zoom-Level
    pub fn zoom_by(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(Self::ZOOM_MIN, Self::ZOOM_MAX);
    }

    /// Gibt die View-Matrix zurück (für Shader)
    /// Enthält nur die Translation. Zoom wird ausschließlich über die Projektion gesteuert.
    pub fn view_matrix(&self) -> Mat3 {
        Mat3::from_translation(-self.position)
    }

    /// Konvertiert Screen-Koordinaten zu Welt-Koordinaten.
    /// Berücksichtigt BASE_WORLD_EXTENT, Zoom und Aspekt-Ratio.
    pub fn screen_to_world(&self, screen_pos: Vec2, screen_size: Vec2) -> Vec2 {
        // Screen-Koordinaten zentrieren (-1 bis 1)
        let ndc = (screen_pos / screen_size) * 2.0 - Vec2::ONE;
        let aspect = screen_size.x / screen_size.y;
        // NDC → Welt: skaliert mit BASE_WORLD_EXTENT / zoom
        // Y nicht negieren – die Projektion flippt Y bereits (bottom=+, top=-)
        Vec2::new(
            ndc.x * Self::BASE_WORLD_EXTENT * aspect / self.zoom,
            ndc.y * Self::BASE_WORLD_EXTENT / self.zoom,
        ) + self.position
    }

    /// Berechnet den Umrechnungsfaktor von Screen-Pixeln zu Welt-Einheiten.
    pub fn world_per_pixel(&self, viewport_height: f32) -> f32 {
        2.0 * Self::BASE_WORLD_EXTENT / (self.zoom * viewport_height)
    }

    /// Berechnet den Pick-Radius in Welt-Einheiten für Node-Selektion.
    ///
    /// Konvertiert den Pixel-Radius in Welt-Koordinaten
    /// basierend auf aktuellem Zoom und Viewport-Höhe.
    pub fn pick_radius_world(&self, viewport_height: f32, pick_radius_px: f32) -> f32 {
        let vh = viewport_height.max(1.0);
        (pick_radius_px * 2.0 * Self::BASE_WORLD_EXTENT) / (self.zoom * vh)
    }

    /// Pick-Radius in Welteinheiten, der beim Rauszoomen auf dem Bildschirm kleiner wird.
    ///
    /// Der Radius wird als fester Welt-Wert berechnet, der bei `ZOOM_MAX` exakt
    /// `pick_radius_px` Screen-Pixeln entspricht. Bei kleinerem Zoom erscheint
    /// derselbe Welt-Radius auf dem Bildschirm proportional kleiner.
    pub fn pick_radius_world_scaled(&self, viewport_height: f32, pick_radius_px: f32) -> f32 {
        let vh = viewport_height.max(1.0);
        // Fester Radius in Welteinheiten = pick_radius_px umgerechnet bei ZOOM_MAX
        (pick_radius_px * 2.0 * Self::BASE_WORLD_EXTENT) / (Self::ZOOM_MAX * vh)
    }
}

impl Default for Camera2D {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_camera_pan() {
        let mut camera = Camera2D::new();
        camera.pan(Vec2::new(10.0, 5.0));
        assert_relative_eq!(camera.position.x, 10.0);
        assert_relative_eq!(camera.position.y, 5.0);
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = Camera2D::new();
        camera.zoom_by(2.0);
        assert_relative_eq!(camera.zoom, 2.0);

        camera.zoom_by(0.5);
        assert_relative_eq!(camera.zoom, 1.0);
    }

    #[test]
    fn test_view_matrix_has_no_scale() {
        let mut camera = Camera2D::new();
        camera.position = Vec2::new(100.0, 50.0);
        camera.zoom = 3.0;
        let mat = camera.view_matrix();
        // Spalten dürfen nur Translation enthalten, kein Zoom-Scale
        assert_relative_eq!(mat.x_axis.x, 1.0);
        assert_relative_eq!(mat.y_axis.y, 1.0);
        assert_relative_eq!(mat.z_axis.x, -100.0);
        assert_relative_eq!(mat.z_axis.y, -50.0);
    }

    #[test]
    fn test_screen_to_world_center() {
        let camera = Camera2D::new(); // pos=0, zoom=1
        let screen_size = Vec2::new(800.0, 600.0);
        // Bildschirm-Mitte → Welt-Ursprung
        let world = camera.screen_to_world(Vec2::new(400.0, 300.0), screen_size);
        assert_relative_eq!(world.x, 0.0, epsilon = 1.0);
        assert_relative_eq!(world.y, 0.0, epsilon = 1.0);
    }

    #[test]
    fn test_screen_to_world_zoom_scales_correctly() {
        let cam1 = Camera2D::new();
        let mut cam2 = Camera2D::new();
        cam2.zoom = 2.0;
        let screen_size = Vec2::new(800.0, 600.0);
        let corner = Vec2::new(800.0, 600.0);
        let w1 = cam1.screen_to_world(corner, screen_size);
        let w2 = cam2.screen_to_world(corner, screen_size);
        // Bei doppeltem Zoom soll der sichtbare Bereich halb so groß sein
        assert_relative_eq!(w2.x, w1.x / 2.0, epsilon = 1.0);
        assert_relative_eq!(w2.y, w1.y / 2.0, epsilon = 1.0);
    }

    #[test]
    fn test_world_per_pixel() {
        let mut camera = Camera2D::new();
        let wpp1 = camera.world_per_pixel(600.0);
        camera.zoom = 2.0;
        let wpp2 = camera.world_per_pixel(600.0);
        // Doppelter Zoom → halb so viele Welt-Einheiten pro Pixel
        assert_relative_eq!(wpp2, wpp1 / 2.0);
    }
}
