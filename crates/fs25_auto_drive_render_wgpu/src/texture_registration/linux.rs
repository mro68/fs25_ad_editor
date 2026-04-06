//! Linux-spezifische DMA-BUF-Descriptorfamilie fuer Texture-Registration-v4.
//!
//! Der aktuelle Repo-Pfad erzeugt normale `wgpu::Texture`-Renderziele ohne
//! External-Memory-/DMA-BUF-Descriptoren. In `wgpu 29` fehlen im oeffentlichen
//! `TextureDescriptor` Export-Felder; `wgpu-hal` stellt zwar rohe Vulkan-Handles
//! bereit, aber dieser Repo-Code allokiert keine exportierbare Device-Memory und
//! erzeugt keine DMA-BUF-FDs oder Modifier fuer das Renderziel. Ein produktiver
//! Linux-v4-Pfad braucht daher backend-spezifische Vulkan-Exportlogik und einen
//! nativen Host-Importpfad.

use super::types::{
    TextureRegistrationAvailability, TextureRegistrationModel, TextureRegistrationPayloadFamily,
    TextureRegistrationPlatform, TextureRegistrationPlatformCapabilities,
};

/// Maximale Anzahl von DMA-BUF-Planes im v4-Basisvertrag.
pub const MAX_LINUX_DMABUF_PLANES: usize = 4;

/// Metadaten einer einzelnen DMA-BUF-Plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LinuxDmabufPlane {
    /// Dateideskriptor der Plane.
    pub fd: i32,
    /// Offset der Plane in Bytes.
    pub offset_bytes: u32,
    /// Zeilen-Stride der Plane in Bytes.
    pub stride_bytes: u32,
}

impl LinuxDmabufPlane {
    /// Erstellt eine neue Plane-Beschreibung.
    pub fn new(fd: i32, offset_bytes: u32, stride_bytes: u32) -> Self {
        Self {
            fd,
            offset_bytes,
            stride_bytes,
        }
    }
}

/// Linux-DMA-BUF-Descriptorfamilie des v4-Vertrags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinuxDmabufDescriptor {
    /// DRM FourCC-Format.
    pub drm_fourcc: u32,
    /// DRM-Modifier (Upper/Lower werden im FFI als `u32` gesplittet).
    pub drm_modifier: u64,
    /// Anzahl gueltiger Planes in `planes`.
    pub plane_count: u32,
    /// Plane-Liste mit fixer ABI-Groesse.
    pub planes: [LinuxDmabufPlane; MAX_LINUX_DMABUF_PLANES],
}

impl LinuxDmabufDescriptor {
    /// Erstellt einen Descriptor mit genau einer Plane.
    pub fn single_plane(drm_fourcc: u32, drm_modifier: u64, plane: LinuxDmabufPlane) -> Self {
        let mut planes = [LinuxDmabufPlane::default(); MAX_LINUX_DMABUF_PLANES];
        planes[0] = plane;

        Self {
            drm_fourcc,
            drm_modifier,
            plane_count: 1,
            planes,
        }
    }
}

/// Liefert die Linux-Capabilities fuer den v4-Vertrag.
pub fn capabilities() -> TextureRegistrationPlatformCapabilities {
    let availability = if cfg!(target_os = "linux") {
        TextureRegistrationAvailability::NotYetImplemented
    } else {
        TextureRegistrationAvailability::Unsupported
    };

    TextureRegistrationPlatformCapabilities::new(
        TextureRegistrationPlatform::Linux,
        TextureRegistrationModel::ExportLease,
        TextureRegistrationPayloadFamily::LinuxDmabuf,
        availability,
    )
}
