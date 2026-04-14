//! Android-spezifische Descriptorfamilien fuer Texture-Registration-v4.
//!
//! Der aktive Android-v4-Pfad nutzt inzwischen `ExportLease` mit
//! `AHardwareBuffer`-Descriptoren. Die fruehere Surface-Attachment-Familie
//! bleibt zusaetzlich als Legacy-Kompatibilitaet fuer aeltere ABI-Consumer im
//! Code erhalten.

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

/// Descriptor fuer Android-AHardwareBuffer-Export im ExportLease-Modell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AndroidHardwareBufferDescriptor {
    /// Opaker Pointer auf einen `AHardwareBuffer`, bereits `acquire()`-t.
    /// Der Empfaenger muss `AHardwareBuffer_release()` aufrufen.
    pub hardware_buffer_ptr: usize,
}

/// Liefert die Android-Capabilities fuer den v4-Vertrag.
pub fn capabilities() -> TextureRegistrationPlatformCapabilities {
    let availability = if cfg!(target_os = "android") {
        TextureRegistrationAvailability::Supported
    } else {
        TextureRegistrationAvailability::Unsupported
    };

    TextureRegistrationPlatformCapabilities::new(
        TextureRegistrationPlatform::Android,
        TextureRegistrationModel::ExportLease,
        TextureRegistrationPayloadFamily::AndroidHardwareBuffer,
        availability,
    )
}
