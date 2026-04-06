//! Windows-spezifische Descriptorfamilie fuer Texture-Registration-v4.
//!
//! Der aktuelle Repo-Pfad rendert Offscreen-Ziele als regulaere `wgpu::Texture`
//! ueber `wgpu::Device::create_texture`. In `wgpu 29` traegt der oeffentliche
//! `TextureDescriptor` keine Export- oder Shared-Handle-Felder; der vorhandene
//! `as_hal`-Abstieg liefert nur nachtraeglich Zugriff auf ein internes
//! `ID3D12Resource`. Ein produktiver Windows-v4-Pfad braucht deshalb zusaetzlich
//! backend-spezifische Export-Erzeugung und nativen Host-Code fuer DXGI- oder
//! D3D11-Registration.

use super::types::{
    TextureRegistrationAvailability, TextureRegistrationModel, TextureRegistrationPayloadFamily,
    TextureRegistrationPlatform, TextureRegistrationPlatformCapabilities,
};

/// Untertyp des Windows-Descriptors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsDescriptorKind {
    /// Descriptor ueber einen DXGI Shared Handle.
    DxgiSharedHandle,
    /// Descriptor ueber einen `ID3D11Texture2D`-Pfad.
    D3d11Texture2D,
}

/// Windows-spezifischer Registration-Descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsDescriptor {
    /// Untertyp der Descriptorfamilie.
    pub kind: WindowsDescriptorKind,
    /// Rohwert eines DXGI Shared Handles.
    pub dxgi_shared_handle: u64,
    /// Opaque Zeigerwert auf eine `ID3D11Texture2D`-Instanz.
    pub d3d11_texture_ptr: usize,
    /// Opaque Zeigerwert auf das zugehoerige D3D11-Device.
    pub d3d11_device_ptr: usize,
}

impl WindowsDescriptor {
    /// Baut einen Descriptor fuer den DXGI-Shared-Handle-Pfad.
    pub fn dxgi_shared_handle(handle: u64) -> Self {
        Self {
            kind: WindowsDescriptorKind::DxgiSharedHandle,
            dxgi_shared_handle: handle,
            d3d11_texture_ptr: 0,
            d3d11_device_ptr: 0,
        }
    }

    /// Baut einen Descriptor fuer den D3D11-Texture-Pfad.
    pub fn d3d11_texture(texture_ptr: usize, device_ptr: usize) -> Self {
        Self {
            kind: WindowsDescriptorKind::D3d11Texture2D,
            dxgi_shared_handle: 0,
            d3d11_texture_ptr: texture_ptr,
            d3d11_device_ptr: device_ptr,
        }
    }
}

/// Liefert die Windows-Capabilities fuer den v4-Vertrag.
pub fn capabilities() -> TextureRegistrationPlatformCapabilities {
    let availability = if cfg!(target_os = "windows") {
        TextureRegistrationAvailability::NotYetImplemented
    } else {
        TextureRegistrationAvailability::Unsupported
    };

    TextureRegistrationPlatformCapabilities::new(
        TextureRegistrationPlatform::Windows,
        TextureRegistrationModel::ExportLease,
        TextureRegistrationPayloadFamily::WindowsDescriptor,
        availability,
    )
}
