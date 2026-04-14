//! Plattformspezifischer GPU-Texture-Export fuer die Flutter-Integration.
//!
//! Dieses Modul definiert den gemeinsamen Trait [`ExternalTextureExport`] und den
//! plattformspezifischen Deskriptor [`PlatformTextureDescriptor`].
//! Konkrete Implementierungen leben in den plattformspezifischen Submodulen.

#[cfg(all(target_os = "linux", feature = "flutter-linux"))]
pub mod vulkan_linux;

/// Stub-Modul fuer zukuenftige Android-Plattformstuetze.
#[cfg(all(target_os = "android", feature = "flutter-android"))]
pub mod vulkan_android;

/// Stub-Modul fuer zukuenftige Windows-Plattformstuetze.
#[cfg(target_os = "windows")]
pub mod dx12_windows;

/// Plattformspezifischer Texture-Deskriptor fuer den Export an Flutter.
///
/// Jede Variante enthaelt die minimal notwendigen Metadaten, damit das
/// Flutter-seitige Native-Plugin die Texture ohne Pixelkopie importieren kann.
#[derive(Debug, Clone)]
pub enum PlatformTextureDescriptor {
    /// Linux: DMA-BUF File Descriptor fuer Vulkan/Impeller-Import.
    #[cfg(target_os = "linux")]
    LinuxDmaBuf {
        /// Exportierter DMA-BUF-Filedescriptor (Eigentuemer ist der Aufrufer).
        fd: i32,
        /// Breite der Textur in Pixeln.
        width: u32,
        /// Hoehe der Textur in Pixeln.
        height: u32,
        /// Zeilenabstand in Bytes.
        stride: u32,
        /// DRM-Format-Konstante (z.B. `DRM_FORMAT_RGBA8888`).
        format: u32,
        /// DRM-Format-Modifier (z.B. `DRM_FORMAT_MOD_LINEAR`).
        modifier: u64,
    },
}

/// Fehler beim plattformspezifischen Texture-Export.
#[derive(Debug, thiserror::Error)]
pub enum ExternalTextureError {
    /// Die benoetzte Vulkan/API-Erweiterung ist auf diesem Geraet nicht verfuegbar.
    #[error("Vulkan External Memory nicht verfuegbar")]
    ExtensionNotAvailable,
    /// Die Texture konnte nicht erzeugt werden.
    #[error("Texture-Erzeugung fehlgeschlagen: {0}")]
    CreationFailed(String),
    /// Der Export des nativen Handles ist fehlgeschlagen.
    #[error("Export fehlgeschlagen: {0}")]
    ExportFailed(String),
    /// Diese Plattform wird nicht unterstuetzt.
    #[error("Plattform nicht unterstuetzt")]
    PlatformNotSupported,
}

/// Trait fuer plattformspezifischen GPU-Texture-Export an Flutter.
///
/// Implementierungen erzeugen und verwalten eine GPU-Texture, die ohne
/// CPU-Kopie an Flutter uebergeben werden kann. Die Texture wird intern
/// fuer den Render-Pass genutzt; nach dem Rendern wird der native Handle
/// via [`export_descriptor`](ExternalTextureExport::export_descriptor) exportiert.
pub trait ExternalTextureExport {
    /// Erzeugt eine neue exportierbare GPU-Texture.
    ///
    /// # Fehler
    /// Gibt [`ExternalTextureError`] zurueck wenn die Texture nicht erzeugt werden konnte.
    fn create_exportable_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Result<Self, ExternalTextureError>
    where
        Self: Sized;

    /// Exportiert den plattformnativen Handle fuer Flutter.
    ///
    /// # Eigentuemer-Semantik
    /// Der zurueckgegebene [`PlatformTextureDescriptor`] (inklusive eines enthaltenen `fd`
    /// bei [`PlatformTextureDescriptor::LinuxDmaBuf`]) wird an den **Aufrufer uebertragen**.
    /// Der Aufrufer ist verantwortlich fuer `close(fd)` nach der Nutzung. Implementierungen
    /// duerfen intern einen separaten, nicht an den Aufrufer uebertragenen Dateideskriptor
    /// behalten, um spaetere Exportaufrufe erneut bedienen zu koennen.
    ///
    /// # Fehler
    /// Gibt [`ExternalTextureError`] zurueck wenn der Export fehlschlaegt.
    fn export_descriptor(&self) -> Result<PlatformTextureDescriptor, ExternalTextureError>;

    /// Gibt die `wgpu::TextureView` zum Rendern zurueck.
    fn texture_view(&self) -> &wgpu::TextureView;

    /// Gibt die zugrundeliegende `wgpu::Texture` zurueck (fuer Copy-Operationen).
    fn texture(&self) -> &wgpu::Texture;

    /// Passt die Texturgroesse an und erzeugt intern eine neue Texture.
    ///
    /// # Fehler
    /// Gibt [`ExternalTextureError`] zurueck wenn die neue Texture nicht erzeugt werden konnte.
    fn resize(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Result<(), ExternalTextureError>;
}
