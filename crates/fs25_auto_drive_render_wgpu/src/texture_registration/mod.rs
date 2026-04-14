//! Additiver Texture-Registration-v4-Vertrag neben dem legacy Shared-Texture-v3-Pfad.
//!
//! Der Vertrag friert die gemeinsame Capability-/Lifecycle-Seam bereits ein,
//! ersetzt aber noch keinen produktiven externen Host-Interop. Dafuer braucht
//! es zusaetzlich native Export-/Attach-Pfade im Render-Backend und native
//! Import-/Surface-Pfade im jeweiligen Ziel-Host.

mod android;
mod linux;
mod types;
mod windows;

pub use android::{
    AndroidAttachmentKind, AndroidHardwareBufferDescriptor, AndroidSurfaceDescriptor,
};
pub use linux::{LinuxDmabufDescriptor, LinuxDmabufPlane, MAX_LINUX_DMABUF_PLANES};
pub use types::{
    TextureRegistrationAlphaMode, TextureRegistrationAvailability, TextureRegistrationCapabilities,
    TextureRegistrationFrameMetadata, TextureRegistrationLifecycle,
    TextureRegistrationLifecycleError, TextureRegistrationLifecycleState, TextureRegistrationModel,
    TextureRegistrationPayloadFamily, TextureRegistrationPixelFormat, TextureRegistrationPlatform,
    TextureRegistrationPlatformCapabilities, TEXTURE_REGISTRATION_V4_CONTRACT_VERSION,
};
pub use windows::{WindowsDescriptor, WindowsDescriptorKind};

/// Liefert die Runtime-Capabilities des additiven Texture-Registration-v4-Vertrags.
///
/// `NotYetImplemented` bedeutet dabei, dass der ABI-Vertrag zwar stabil ist,
/// fuer produktiven externen Host-Interop aber noch native Backend- oder
/// Host-Pfade fehlen.
pub fn query_texture_registration_v4_capabilities() -> TextureRegistrationCapabilities {
    TextureRegistrationCapabilities {
        contract_version: TEXTURE_REGISTRATION_V4_CONTRACT_VERSION,
        pixel_format: TextureRegistrationPixelFormat::Rgba8Srgb,
        alpha_mode: TextureRegistrationAlphaMode::Premultiplied,
        requires_explicit_release: true,
        windows: windows::capabilities(),
        linux: linux::capabilities(),
        android: android::capabilities(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        query_texture_registration_v4_capabilities, AndroidAttachmentKind,
        AndroidHardwareBufferDescriptor, AndroidSurfaceDescriptor, LinuxDmabufDescriptor,
        LinuxDmabufPlane, TextureRegistrationAvailability, TextureRegistrationLifecycle,
        TextureRegistrationLifecycleError, TextureRegistrationLifecycleState,
        TextureRegistrationModel, TextureRegistrationPayloadFamily, TextureRegistrationPixelFormat,
        TextureRegistrationPlatform, WindowsDescriptor, WindowsDescriptorKind,
        TEXTURE_REGISTRATION_V4_CONTRACT_VERSION,
    };

    #[test]
    fn v4_capabilities_expose_platform_payload_families() {
        let capabilities = query_texture_registration_v4_capabilities();

        assert_eq!(
            capabilities.contract_version,
            TEXTURE_REGISTRATION_V4_CONTRACT_VERSION
        );
        assert_eq!(
            capabilities.pixel_format,
            TextureRegistrationPixelFormat::Rgba8Srgb
        );
        assert!(capabilities.requires_explicit_release);

        let windows = capabilities.platform(TextureRegistrationPlatform::Windows);
        assert_eq!(windows.model, TextureRegistrationModel::ExportLease);
        assert_eq!(
            windows.payload_family,
            TextureRegistrationPayloadFamily::WindowsDescriptor
        );

        let linux = capabilities.platform(TextureRegistrationPlatform::Linux);
        assert_eq!(linux.model, TextureRegistrationModel::ExportLease);
        assert_eq!(
            linux.payload_family,
            TextureRegistrationPayloadFamily::LinuxDmabuf
        );

        let android = capabilities.platform(TextureRegistrationPlatform::Android);
        assert_eq!(android.model, TextureRegistrationModel::ExportLease);
        assert_eq!(
            android.payload_family,
            TextureRegistrationPayloadFamily::AndroidHardwareBuffer
        );
    }

    #[test]
    fn v4_capabilities_report_target_specific_availability() {
        let capabilities = query_texture_registration_v4_capabilities();

        let windows = capabilities
            .platform(TextureRegistrationPlatform::Windows)
            .availability;
        if cfg!(target_os = "windows") {
            assert_eq!(windows, TextureRegistrationAvailability::NotYetImplemented);
        } else {
            assert_eq!(windows, TextureRegistrationAvailability::Unsupported);
        }

        let linux = capabilities
            .platform(TextureRegistrationPlatform::Linux)
            .availability;
        if cfg!(target_os = "linux") {
            assert_eq!(linux, TextureRegistrationAvailability::NotYetImplemented);
        } else {
            assert_eq!(linux, TextureRegistrationAvailability::Unsupported);
        }

        let android = capabilities
            .platform(TextureRegistrationPlatform::Android)
            .availability;
        if cfg!(target_os = "android") {
            assert_eq!(android, TextureRegistrationAvailability::Supported);
        } else {
            assert_eq!(android, TextureRegistrationAvailability::Unsupported);
        }
    }

    #[test]
    fn v4_payload_family_numeric_values_keep_android_legacy_and_ahb_distinct() {
        assert_eq!(
            TextureRegistrationPayloadFamily::WindowsDescriptor.as_u32(),
            1
        );
        assert_eq!(TextureRegistrationPayloadFamily::LinuxDmabuf.as_u32(), 2);
        assert_eq!(
            TextureRegistrationPayloadFamily::AndroidSurfaceAttachment.as_u32(),
            3
        );
        assert_eq!(
            TextureRegistrationPayloadFamily::AndroidHardwareBuffer.as_u32(),
            4
        );
    }

    #[test]
    fn v4_lifecycle_export_model_tracks_render_and_lease_tokens() {
        let mut lifecycle =
            TextureRegistrationLifecycle::new(TextureRegistrationModel::ExportLease);

        let frame = lifecycle
            .record_render(8, 6, 1, 1)
            .expect("render registration must succeed");
        assert_eq!(
            lifecycle.state(),
            TextureRegistrationLifecycleState::FrameAvailable {
                frame_token: frame.frame_token,
            }
        );

        let leased = lifecycle
            .acquire_frame()
            .expect("acquire registration must succeed");
        assert_eq!(leased.frame_token, frame.frame_token);
        assert_eq!(
            lifecycle.state(),
            TextureRegistrationLifecycleState::FrameLeased {
                frame_token: frame.frame_token,
            }
        );

        assert!(matches!(
            lifecycle.record_render(8, 6, 1, 1),
            Err(TextureRegistrationLifecycleError::FrameInUse { frame_token })
                if frame_token == frame.frame_token
        ));
        assert!(matches!(
            lifecycle.on_resize(),
            Err(TextureRegistrationLifecycleError::FrameInUse { frame_token })
                if frame_token == frame.frame_token
        ));

        lifecycle
            .release_frame(frame.frame_token)
            .expect("release must succeed");
        lifecycle
            .on_resize()
            .expect("resize after release must succeed");
        assert_eq!(lifecycle.state(), TextureRegistrationLifecycleState::Idle);
    }

    #[test]
    fn v4_lifecycle_reports_release_token_mismatch() {
        let mut lifecycle =
            TextureRegistrationLifecycle::new(TextureRegistrationModel::ExportLease);

        let frame = lifecycle
            .record_render(8, 6, 9, 2)
            .expect("render registration must succeed");
        let leased = lifecycle
            .acquire_frame()
            .expect("acquire registration must succeed");

        assert!(matches!(
            lifecycle.release_frame(leased.frame_token + 1),
            Err(TextureRegistrationLifecycleError::FrameLeaseMismatch {
                expected,
                actual,
            }) if expected == leased.frame_token && actual == leased.frame_token + 1
        ));

        lifecycle
            .release_frame(frame.frame_token)
            .expect("release with matching token must succeed");
    }

    #[test]
    fn v4_lifecycle_requires_android_attach_before_render() {
        let mut lifecycle =
            TextureRegistrationLifecycle::new(TextureRegistrationModel::HostAttachedSurface);

        assert!(matches!(
            lifecycle.record_render(8, 6, 1, 1),
            Err(TextureRegistrationLifecycleError::SurfaceNotAttached)
        ));

        lifecycle
            .attach_surface()
            .expect("attach must succeed for android model");
        let frame = lifecycle
            .record_render(8, 6, 1, 1)
            .expect("render after attach must succeed");
        let leased = lifecycle.acquire_frame().expect("acquire must succeed");

        assert!(matches!(
            lifecycle.detach_surface(),
            Err(TextureRegistrationLifecycleError::SurfaceDetachWhileLeased {
                frame_token,
            }) if frame_token == leased.frame_token
        ));

        lifecycle
            .release_frame(frame.frame_token)
            .expect("release must succeed");
        lifecycle
            .detach_surface()
            .expect("detach after release must succeed");
        assert_eq!(lifecycle.state(), TextureRegistrationLifecycleState::Idle);
        assert!(!lifecycle.is_surface_attached());
    }

    #[test]
    fn v4_payload_families_keep_platform_specific_shapes() {
        let windows = WindowsDescriptor::dxgi_shared_handle(0x44);
        assert_eq!(windows.kind, WindowsDescriptorKind::DxgiSharedHandle);
        assert_eq!(windows.dxgi_shared_handle, 0x44);

        let plane = LinuxDmabufPlane::new(7, 128, 256);
        let linux = LinuxDmabufDescriptor::single_plane(0x34325258, 0x0102_0304_0506_0708, plane);
        assert_eq!(linux.plane_count, 1);
        assert_eq!(linux.planes[0], plane);

        let android = AndroidSurfaceDescriptor::for_surface_producer(0x11, 0x22);
        assert_eq!(
            android.attachment_kind,
            AndroidAttachmentKind::SurfaceProducer
        );
        assert_eq!(android.native_window_ptr, 0x11);
        assert_eq!(android.surface_handle_ptr, 0x22);

        let ahb = AndroidHardwareBufferDescriptor {
            hardware_buffer_ptr: 0x33,
        };
        assert_eq!(ahb.hardware_buffer_ptr, 0x33);
    }
}
