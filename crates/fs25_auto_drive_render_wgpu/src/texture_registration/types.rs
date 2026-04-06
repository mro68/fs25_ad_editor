//! Gemeinsame Typen und Lifecycle-Regeln fuer den additiven Texture-Registration-v4-Vertrag.

use std::fmt;

/// Vertragsversion des additiven Texture-Registration-v4-Vertrags.
pub const TEXTURE_REGISTRATION_V4_CONTRACT_VERSION: u32 = 4;

/// Zielplattform eines plattformspezifischen Registration-Payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureRegistrationPlatform {
    /// Windows-Host mit descriptor-basiertem Exportmodell.
    Windows,
    /// Linux-Host mit DMA-BUF-Descriptorfamilie.
    Linux,
    /// Android-Host mit Surface-Attach-Modell.
    Android,
}

impl fmt::Display for TextureRegistrationPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Windows => write!(f, "windows"),
            Self::Linux => write!(f, "linux"),
            Self::Android => write!(f, "android"),
        }
    }
}

/// Registrierungsmodell der jeweiligen Plattformfamilie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureRegistrationModel {
    /// Exportierter Handle-/Descriptor-Pfad mit Acquire/Release-Lease.
    ExportLease,
    /// Host-attached Surface-Pfad mit explizitem Attach/Detach.
    HostAttachedSurface,
}

/// Payload-Familie einer Plattform im v4-Vertrag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureRegistrationPayloadFamily {
    /// Windows-Descriptorfamilie (z. B. DXGI / D3D11).
    WindowsDescriptor,
    /// Linux-DMA-BUF-Descriptorfamilie.
    LinuxDmabuf,
    /// Android-Surface-Attachment-Familie.
    AndroidSurfaceAttachment,
}

/// Verfuegbarkeitsstatus eines Plattformpfads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureRegistrationAvailability {
    /// Der Plattformpfad ist in diesem Build produktiv nutzbar.
    Supported,
    /// Der Plattformpfad ist auf dem Target prinzipiell vorgesehen, aber noch nicht implementiert.
    /// Typische Gruende sind fehlende backend-native Export-/Attach-Pfade oder
    /// fehlende native Host-Import-/Surface-Pfade.
    NotYetImplemented,
    /// Der Plattformpfad ist auf dem aktuellen Target nicht verfuegbar.
    Unsupported,
}

/// Exportiertes Pixel-Format eines registrierten Frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureRegistrationPixelFormat {
    /// `RGBA8` im sRGB-Farbraum.
    Rgba8Srgb,
}

/// Exportierter Alpha-Modus eines registrierten Frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureRegistrationAlphaMode {
    /// Farbwerte sind premultiplied gespeichert.
    Premultiplied,
}

/// Plattformspezifische Capabilities eines einzelnen v4-Pfads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureRegistrationPlatformCapabilities {
    /// Zielplattform der Payload-Familie.
    pub platform: TextureRegistrationPlatform,
    /// Registrierungsmodell der Plattform.
    pub model: TextureRegistrationModel,
    /// Payload-Familie der Plattform.
    pub payload_family: TextureRegistrationPayloadFamily,
    /// Verfuegbarkeitsstatus auf dem aktuellen Build-Target.
    pub availability: TextureRegistrationAvailability,
}

impl TextureRegistrationPlatformCapabilities {
    /// Erstellt einen Plattform-Capability-Eintrag.
    pub fn new(
        platform: TextureRegistrationPlatform,
        model: TextureRegistrationModel,
        payload_family: TextureRegistrationPayloadFamily,
        availability: TextureRegistrationAvailability,
    ) -> Self {
        Self {
            platform,
            model,
            payload_family,
            availability,
        }
    }
}

/// Gemeinsame Runtime-Capabilities des additiven v4-Vertrags.
///
/// Die Matrix beschreibt den stabilen ABI-Vertrag. Sie bedeutet noch keinen
/// produktiven externen Host-Pfad, solange `availability` auf fehlende native
/// Backend- oder Host-Integrationen verweist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureRegistrationCapabilities {
    /// Vertragsversion.
    pub contract_version: u32,
    /// Gemeinsames Pixel-Format.
    pub pixel_format: TextureRegistrationPixelFormat,
    /// Gemeinsamer Alpha-Modus.
    pub alpha_mode: TextureRegistrationAlphaMode,
    /// `true`, wenn Acquire/Release vom Host explizit eingehalten werden muss.
    pub requires_explicit_release: bool,
    /// Windows-Pfad.
    pub windows: TextureRegistrationPlatformCapabilities,
    /// Linux-Pfad.
    pub linux: TextureRegistrationPlatformCapabilities,
    /// Android-Pfad.
    pub android: TextureRegistrationPlatformCapabilities,
}

impl TextureRegistrationCapabilities {
    /// Liefert den Capability-Eintrag fuer eine einzelne Plattform.
    pub fn platform(
        &self,
        platform: TextureRegistrationPlatform,
    ) -> TextureRegistrationPlatformCapabilities {
        match platform {
            TextureRegistrationPlatform::Windows => self.windows,
            TextureRegistrationPlatform::Linux => self.linux,
            TextureRegistrationPlatform::Android => self.android,
        }
    }
}

/// Gemeinsame Frame-Metadaten fuer alle v4-Plattformfamilien.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureRegistrationFrameMetadata {
    /// Frame-Breite in Pixeln.
    pub width: u32,
    /// Frame-Hoehe in Pixeln.
    pub height: u32,
    /// Exportiertes Pixel-Format.
    pub pixel_format: TextureRegistrationPixelFormat,
    /// Exportierter Alpha-Modus.
    pub alpha_mode: TextureRegistrationAlphaMode,
    /// Runtime-ID der zugrundeliegenden Textur.
    pub texture_id: u64,
    /// Generation der Textur.
    pub texture_generation: u64,
    /// Lease-Token fuer Acquire/Release.
    pub frame_token: u64,
}

/// Sichtbarer Lifecycle-State eines v4-Registrationspfads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureRegistrationLifecycleState {
    /// Es existiert noch kein gerenderter Frame oder der letzte wurde invalidiert.
    Idle,
    /// Ein Frame liegt vor und kann geleast werden.
    FrameAvailable { frame_token: u64 },
    /// Ein Frame ist aktuell geleast.
    FrameLeased { frame_token: u64 },
}

/// Fehler des v4-Lifecycle-Guards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureRegistrationLifecycleError {
    /// Es wurde noch kein Frame gerendert.
    FrameUnavailable,
    /// Ein Frame ist bereits geleast.
    FrameAlreadyAcquired { frame_token: u64 },
    /// Es existiert kein aktiver Lease.
    FrameLeaseMissing,
    /// Das uebergebene Token passt nicht zum aktiven Lease.
    FrameLeaseMismatch { expected: u64, actual: u64 },
    /// Render/Resize ist waehrend eines aktiven Leases nicht erlaubt.
    FrameInUse { frame_token: u64 },
    /// Der Android-Surface-Pfad wurde noch nicht attached.
    SurfaceNotAttached,
    /// Der Android-Surface-Pfad ist bereits attached.
    SurfaceAlreadyAttached,
    /// Ein Surface darf bei aktivem Lease nicht detached werden.
    SurfaceDetachWhileLeased { frame_token: u64 },
}

impl fmt::Display for TextureRegistrationLifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrameUnavailable => write!(f, "texture registration frame is unavailable"),
            Self::FrameAlreadyAcquired { frame_token } => write!(
                f,
                "texture registration frame {frame_token} is already acquired"
            ),
            Self::FrameLeaseMissing => write!(f, "texture registration frame is not acquired"),
            Self::FrameLeaseMismatch { expected, actual } => {
                write!(f, "texture registration token mismatch: expected {expected}, got {actual}")
            }
            Self::FrameInUse { frame_token } => write!(
                f,
                "texture registration frame {frame_token} is acquired; release before render or resize"
            ),
            Self::SurfaceNotAttached => write!(f, "android surface must be attached before render"),
            Self::SurfaceAlreadyAttached => write!(f, "android surface is already attached"),
            Self::SurfaceDetachWhileLeased { frame_token } => write!(
                f,
                "android surface cannot detach while frame {frame_token} is acquired"
            ),
        }
    }
}

impl std::error::Error for TextureRegistrationLifecycleError {}

/// Modellunabhaengiger Lifecycle-Guard fuer Acquire/Release, Resize und Android Attach/Detach.
#[derive(Debug, Clone)]
pub struct TextureRegistrationLifecycle {
    model: TextureRegistrationModel,
    next_frame_token: u64,
    last_frame: Option<TextureRegistrationFrameMetadata>,
    acquired_frame_token: Option<u64>,
    surface_attached: bool,
}

impl TextureRegistrationLifecycle {
    /// Erstellt einen neuen Lifecycle-Guard fuer das gewaehlte Modell.
    pub fn new(model: TextureRegistrationModel) -> Self {
        let surface_attached = matches!(model, TextureRegistrationModel::ExportLease);

        Self {
            model,
            next_frame_token: 1,
            last_frame: None,
            acquired_frame_token: None,
            surface_attached,
        }
    }

    /// Liefert den aktuellen Lifecycle-State.
    pub fn state(&self) -> TextureRegistrationLifecycleState {
        if let Some(frame_token) = self.acquired_frame_token {
            return TextureRegistrationLifecycleState::FrameLeased { frame_token };
        }

        if let Some(frame) = self.last_frame {
            return TextureRegistrationLifecycleState::FrameAvailable {
                frame_token: frame.frame_token,
            };
        }

        TextureRegistrationLifecycleState::Idle
    }

    /// Liefert, ob ein Android-Surface aktuell attached ist.
    pub fn is_surface_attached(&self) -> bool {
        self.surface_attached
    }

    /// Markiert ein Android-Surface als attached.
    pub fn attach_surface(&mut self) -> Result<(), TextureRegistrationLifecycleError> {
        if !matches!(self.model, TextureRegistrationModel::HostAttachedSurface) {
            return Ok(());
        }

        if self.surface_attached {
            return Err(TextureRegistrationLifecycleError::SurfaceAlreadyAttached);
        }

        self.surface_attached = true;
        Ok(())
    }

    /// Markiert ein Android-Surface als detached.
    pub fn detach_surface(&mut self) -> Result<(), TextureRegistrationLifecycleError> {
        if !matches!(self.model, TextureRegistrationModel::HostAttachedSurface) {
            return Ok(());
        }

        if let Some(frame_token) = self.acquired_frame_token {
            return Err(
                TextureRegistrationLifecycleError::SurfaceDetachWhileLeased { frame_token },
            );
        }

        if !self.surface_attached {
            return Err(TextureRegistrationLifecycleError::SurfaceNotAttached);
        }

        self.surface_attached = false;
        self.last_frame = None;
        Ok(())
    }

    /// Registriert einen neuen gerenderten Frame.
    pub fn record_render(
        &mut self,
        width: u32,
        height: u32,
        texture_id: u64,
        texture_generation: u64,
    ) -> Result<TextureRegistrationFrameMetadata, TextureRegistrationLifecycleError> {
        self.ensure_not_acquired()?;

        if matches!(self.model, TextureRegistrationModel::HostAttachedSurface)
            && !self.surface_attached
        {
            return Err(TextureRegistrationLifecycleError::SurfaceNotAttached);
        }

        let frame = TextureRegistrationFrameMetadata {
            width,
            height,
            pixel_format: TextureRegistrationPixelFormat::Rgba8Srgb,
            alpha_mode: TextureRegistrationAlphaMode::Premultiplied,
            texture_id,
            texture_generation,
            frame_token: self.next_frame_token(),
        };

        self.last_frame = Some(frame);
        Ok(frame)
    }

    /// Leased den zuletzt registrierten Frame.
    pub fn acquire_frame(
        &mut self,
    ) -> Result<TextureRegistrationFrameMetadata, TextureRegistrationLifecycleError> {
        if let Some(frame_token) = self.acquired_frame_token {
            return Err(TextureRegistrationLifecycleError::FrameAlreadyAcquired { frame_token });
        }

        let frame = self
            .last_frame
            .ok_or(TextureRegistrationLifecycleError::FrameUnavailable)?;
        self.acquired_frame_token = Some(frame.frame_token);
        Ok(frame)
    }

    /// Gibt einen aktiven Lease wieder frei.
    pub fn release_frame(
        &mut self,
        frame_token: u64,
    ) -> Result<(), TextureRegistrationLifecycleError> {
        match self.acquired_frame_token {
            Some(active_token) if active_token == frame_token => {
                self.acquired_frame_token = None;
                Ok(())
            }
            Some(active_token) => Err(TextureRegistrationLifecycleError::FrameLeaseMismatch {
                expected: active_token,
                actual: frame_token,
            }),
            None => Err(TextureRegistrationLifecycleError::FrameLeaseMissing),
        }
    }

    /// Invalidiert den zuletzt gerenderten Frame nach Resize/Recreate.
    pub fn on_resize(&mut self) -> Result<(), TextureRegistrationLifecycleError> {
        self.ensure_not_acquired()?;
        self.last_frame = None;
        Ok(())
    }

    fn ensure_not_acquired(&self) -> Result<(), TextureRegistrationLifecycleError> {
        if let Some(frame_token) = self.acquired_frame_token {
            return Err(TextureRegistrationLifecycleError::FrameInUse { frame_token });
        }

        Ok(())
    }

    fn next_frame_token(&mut self) -> u64 {
        let token = self.next_frame_token;
        self.next_frame_token = if token == u64::MAX { 1 } else { token + 1 };
        token
    }
}
