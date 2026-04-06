//! Android-spezifische Surface-Attachment-Familie fuer Texture-Registration-v4.
//!
//! Der aktuelle Repo-Pfad rendert Offscreen immer in eine interne
//! `wgpu::Texture`. Fuer produktives Android-v4-Interop reicht das nicht: der
//! Host muss ein `ANativeWindow`/Surface-Ziel bereitstellen, und der Renderer
//! muss backend-spezifisch gegen dieses Ziel statt gegen eine interne
//! Offscreen-Textur rendern. Ohne nativen Host-Code und ohne niedrigeren
//! Surface-/Swapchain-Pfad im Backend kann dieser Vertrag hier nicht produktiv
//! eingelost werden.

use super::types::{
    TextureRegistrationAvailability, TextureRegistrationModel, TextureRegistrationPayloadFamily,
    TextureRegistrationPlatform, TextureRegistrationPlatformCapabilities,
};

/// Untertyp der Android-Surface-Attachment-Familie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidAttachmentKind {
    /// Host-attached `ANativeWindow`-Pfad.
    NativeWindow,
    /// Host-attached SurfaceProducer-Pfad.
    SurfaceProducer,
}

/// Android-spezifische Surface-Beschreibung fuer den v4-Vertrag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AndroidSurfaceDescriptor {
    /// Untertyp des Android-Attachment-Pfads.
    pub attachment_kind: AndroidAttachmentKind,
    /// Opaque Zeigerwert auf ein `ANativeWindow`-Aequivalent.
    pub native_window_ptr: usize,
    /// Opaque Zeigerwert auf ein hostseitiges Surface-Objekt.
    pub surface_handle_ptr: usize,
}

impl AndroidSurfaceDescriptor {
    /// Erstellt eine Surface-Beschreibung fuer den NativeWindow-Pfad.
    pub fn for_native_window(native_window_ptr: usize) -> Self {
        Self {
            attachment_kind: AndroidAttachmentKind::NativeWindow,
            native_window_ptr,
            surface_handle_ptr: 0,
        }
    }

    /// Erstellt eine Surface-Beschreibung fuer den SurfaceProducer-Pfad.
    pub fn for_surface_producer(native_window_ptr: usize, surface_handle_ptr: usize) -> Self {
        Self {
            attachment_kind: AndroidAttachmentKind::SurfaceProducer,
            native_window_ptr,
            surface_handle_ptr,
        }
    }
}

/// Liefert die Android-Capabilities fuer den v4-Vertrag.
pub fn capabilities() -> TextureRegistrationPlatformCapabilities {
    let availability = if cfg!(target_os = "android") {
        TextureRegistrationAvailability::NotYetImplemented
    } else {
        TextureRegistrationAvailability::Unsupported
    };

    TextureRegistrationPlatformCapabilities::new(
        TextureRegistrationPlatform::Android,
        TextureRegistrationModel::HostAttachedSurface,
        TextureRegistrationPayloadFamily::AndroidSurfaceAttachment,
        availability,
    )
}
