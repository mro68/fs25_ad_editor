#ifndef FS25AD_HOST_BRIDGE_H
#define FS25AD_HOST_BRIDGE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define FS25AD_HOST_BRIDGE_ABI_VERSION 4u
#define FS25AD_HOST_BRIDGE_SHARED_TEXTURE_CONTRACT_VERSION 3u
#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_CONTRACT_VERSION 4u

#define FS25AD_HOST_BRIDGE_SHARED_TEXTURE_PIXEL_FORMAT_RGBA8_SRGB 1u
#define FS25AD_HOST_BRIDGE_SHARED_TEXTURE_ALPHA_MODE_PREMULTIPLIED 1u
#define FS25AD_HOST_BRIDGE_SHARED_TEXTURE_NATIVE_HANDLE_OPAQUE_RUNTIME_POINTERS 1u
#define FS25AD_HOST_BRIDGE_SHARED_TEXTURE_NATIVE_HANDLE_WGPU_POINTERS \
    FS25AD_HOST_BRIDGE_SHARED_TEXTURE_NATIVE_HANDLE_OPAQUE_RUNTIME_POINTERS

#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_PIXEL_FORMAT_RGBA8_SRGB 1u
#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_ALPHA_MODE_PREMULTIPLIED 1u

#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_PLATFORM_WINDOWS 1u
#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_PLATFORM_LINUX 2u
#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_PLATFORM_ANDROID 3u

#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_MODEL_EXPORT_LEASE 1u
#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_MODEL_HOST_ATTACHED_SURFACE 2u

#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_PAYLOAD_WINDOWS_DESCRIPTOR 1u
#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_PAYLOAD_LINUX_DMABUF 2u
/* Legacy */
#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_PAYLOAD_ANDROID_SURFACE_ATTACHMENT 3u
/* Android AHardwareBuffer ExportLease (Payload-Familie 4) */
#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_PAYLOAD_ANDROID_HARDWARE_BUFFER 4u

#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_AVAILABILITY_SUPPORTED 1u
#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_AVAILABILITY_NOT_YET_IMPLEMENTED 2u
#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_AVAILABILITY_UNSUPPORTED 3u

#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_WINDOWS_DESCRIPTOR_DXGI_SHARED_HANDLE 1u
#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_WINDOWS_DESCRIPTOR_D3D11_TEXTURE2D 2u

/* Legacy */
#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_ANDROID_ATTACHMENT_NATIVE_WINDOW 1u
/* Legacy */
#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_ANDROID_ATTACHMENT_SURFACE_PRODUCER 2u

#define FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_MAX_LINUX_DMABUF_PLANES 4u

typedef struct Fs25adHostBridgeSession Fs25adHostBridgeSession;
typedef struct Fs25adHostBridgeSharedTexture Fs25adHostBridgeSharedTexture;
typedef struct Fs25adHostBridgeTextureRegistrationV4
    Fs25adHostBridgeTextureRegistrationV4;
typedef struct Fs25adFlutterSessionHandle Fs25adFlutterSessionHandle;
typedef struct Fs25adGpuRuntimeHandle Fs25adGpuRuntimeHandle;

typedef struct Fs25adSharedTextureCapabilities {
    uint32_t pixel_format;
    uint32_t alpha_mode;
    uint32_t native_handle_kind;
    uint32_t requires_explicit_release;
} Fs25adSharedTextureCapabilities;

typedef struct Fs25adSharedTextureFrameInfo {
    uint32_t width;
    uint32_t height;
    uint32_t pixel_format;
    uint32_t alpha_mode;
    uint64_t texture_id;
    uint64_t texture_generation;
    uint64_t frame_token;
} Fs25adSharedTextureFrameInfo;

typedef struct Fs25adSharedTextureNativeHandle {
    /*
     * Opaque Pointerwerte auf Rust/wgpu-Runtimeobjekte im selben Prozessraum.
     * Keine backend-nativen Interop-Handles fuer Vulkan/Metal/DX.
     */
    uintptr_t texture_ptr;
    uintptr_t texture_view_ptr;
} Fs25adSharedTextureNativeHandle;

typedef struct Fs25adTextureRegistrationV4PlatformCapabilities {
    uint32_t platform;
    uint32_t registration_model;
    uint32_t payload_family;
    uint32_t availability;
} Fs25adTextureRegistrationV4PlatformCapabilities;

typedef struct Fs25adTextureRegistrationV4Capabilities {
    uint32_t contract_version;
    uint32_t pixel_format;
    uint32_t alpha_mode;
    uint32_t requires_explicit_release;
    Fs25adTextureRegistrationV4PlatformCapabilities windows;
    Fs25adTextureRegistrationV4PlatformCapabilities linux;
    Fs25adTextureRegistrationV4PlatformCapabilities android;
} Fs25adTextureRegistrationV4Capabilities;

typedef struct Fs25adTextureRegistrationV4FrameInfo {
    uint32_t width;
    uint32_t height;
    uint32_t pixel_format;
    uint32_t alpha_mode;
    uint64_t texture_id;
    uint64_t texture_generation;
    uint64_t frame_token;
} Fs25adTextureRegistrationV4FrameInfo;

typedef struct Fs25adTextureRegistrationV4WindowsDescriptor {
    uint32_t descriptor_kind;
    uint64_t dxgi_shared_handle;
    uintptr_t d3d11_texture_ptr;
    uintptr_t d3d11_device_ptr;
} Fs25adTextureRegistrationV4WindowsDescriptor;

typedef struct Fs25adTextureRegistrationV4LinuxDmabufPlane {
    int32_t fd;
    uint32_t offset_bytes;
    uint32_t stride_bytes;
} Fs25adTextureRegistrationV4LinuxDmabufPlane;

typedef struct Fs25adTextureRegistrationV4LinuxDmabufDescriptor {
    uint32_t drm_fourcc;
    uint32_t drm_modifier_hi;
    uint32_t drm_modifier_lo;
    uint32_t plane_count;
    Fs25adTextureRegistrationV4LinuxDmabufPlane
        planes[FS25AD_HOST_BRIDGE_TEXTURE_REGISTRATION_V4_MAX_LINUX_DMABUF_PLANES];
} Fs25adTextureRegistrationV4LinuxDmabufDescriptor;

typedef struct Fs25adTextureRegistrationV4AndroidHardwareBufferDescriptor {
    uintptr_t hardware_buffer_ptr;
} Fs25adTextureRegistrationV4AndroidHardwareBufferDescriptor;

/* Legacy */
typedef struct Fs25adTextureRegistrationV4AndroidSurfaceDescriptor {
    uint32_t attachment_kind;
    uintptr_t native_window_ptr;
    uintptr_t surface_handle_ptr;
} Fs25adTextureRegistrationV4AndroidSurfaceDescriptor;

uint32_t fs25ad_host_bridge_abi_version(void);
uint32_t fs25ad_host_bridge_shared_texture_contract_version(void);
bool fs25ad_host_bridge_shared_texture_capabilities(
    Fs25adSharedTextureCapabilities *out_capabilities);

char *fs25ad_host_bridge_last_error_message(void);
void fs25ad_host_bridge_string_free(char *value);

Fs25adHostBridgeSession *fs25ad_host_bridge_session_new(void);
void fs25ad_host_bridge_session_dispose(Fs25adHostBridgeSession *session);

char *fs25ad_host_bridge_session_snapshot_json(Fs25adHostBridgeSession *session);
char *fs25ad_host_bridge_session_chrome_snapshot_json(Fs25adHostBridgeSession *session);
char *fs25ad_host_bridge_session_node_details_json(Fs25adHostBridgeSession *session);
char *fs25ad_host_bridge_session_marker_list_json(Fs25adHostBridgeSession *session);
char *fs25ad_host_bridge_session_connection_pair_json(
    Fs25adHostBridgeSession *session,
    uint64_t node_a,
    uint64_t node_b);
int32_t fs25ad_host_bridge_session_is_dirty(Fs25adHostBridgeSession *session);
char *fs25ad_host_bridge_session_ui_snapshot_json(Fs25adHostBridgeSession *session);
char *fs25ad_host_bridge_session_dialog_snapshot_json(Fs25adHostBridgeSession *session);
char *fs25ad_host_bridge_session_editing_snapshot_json(Fs25adHostBridgeSession *session);
char *fs25ad_host_bridge_session_context_menu_snapshot_json(
    Fs25adHostBridgeSession *session,
    int64_t focus_node_id_or_neg1);
char *fs25ad_host_bridge_session_route_tool_viewport_json(
    Fs25adHostBridgeSession *session);
char *fs25ad_host_bridge_session_viewport_overlay_json(
    Fs25adHostBridgeSession *session,
    float cursor_world_x,
    float cursor_world_y);
bool fs25ad_host_bridge_session_apply_action_json(
    Fs25adHostBridgeSession *session,
    const char *action_json);
char *fs25ad_host_bridge_session_take_dialog_requests_json(Fs25adHostBridgeSession *session);
bool fs25ad_host_bridge_session_submit_dialog_result_json(
    Fs25adHostBridgeSession *session,
    const char *result_json);
char *fs25ad_host_bridge_session_viewport_geometry_json(
    Fs25adHostBridgeSession *session,
    float viewport_width,
    float viewport_height);

Fs25adHostBridgeSharedTexture *fs25ad_host_bridge_shared_texture_new(
    uint32_t width,
    uint32_t height);
void fs25ad_host_bridge_shared_texture_dispose(
    Fs25adHostBridgeSharedTexture *texture);
bool fs25ad_host_bridge_shared_texture_resize(
    Fs25adHostBridgeSharedTexture *texture,
    uint32_t width,
    uint32_t height);
bool fs25ad_host_bridge_shared_texture_render(
    Fs25adHostBridgeSession *session,
    Fs25adHostBridgeSharedTexture *texture);
bool fs25ad_host_bridge_shared_texture_acquire(
    Fs25adHostBridgeSharedTexture *texture,
    Fs25adSharedTextureFrameInfo *out_frame_info,
    Fs25adSharedTextureNativeHandle *out_native_handle);
bool fs25ad_host_bridge_shared_texture_release(
    Fs25adHostBridgeSharedTexture *texture,
    uint64_t frame_token);

uint32_t fs25ad_host_bridge_texture_registration_v4_contract_version(void);
bool fs25ad_host_bridge_texture_registration_v4_capabilities(
    Fs25adTextureRegistrationV4Capabilities *out_capabilities);

Fs25adHostBridgeTextureRegistrationV4 *fs25ad_host_bridge_texture_registration_v4_new(
    uint32_t platform,
    uint32_t width,
    uint32_t height);
void fs25ad_host_bridge_texture_registration_v4_dispose(
    Fs25adHostBridgeTextureRegistrationV4 *texture);
bool fs25ad_host_bridge_texture_registration_v4_resize(
    Fs25adHostBridgeTextureRegistrationV4 *texture,
    uint32_t width,
    uint32_t height);
bool fs25ad_host_bridge_texture_registration_v4_render(
    Fs25adHostBridgeSession *session,
    Fs25adHostBridgeTextureRegistrationV4 *texture);
bool fs25ad_host_bridge_texture_registration_v4_acquire(
    Fs25adHostBridgeTextureRegistrationV4 *texture,
    Fs25adTextureRegistrationV4FrameInfo *out_frame_info);
bool fs25ad_host_bridge_texture_registration_v4_release(
    Fs25adHostBridgeTextureRegistrationV4 *texture,
    uint64_t frame_token);
bool fs25ad_host_bridge_texture_registration_v4_get_windows_descriptor(
    Fs25adHostBridgeTextureRegistrationV4 *texture,
    uint64_t frame_token,
    Fs25adTextureRegistrationV4WindowsDescriptor *out_descriptor);
bool fs25ad_host_bridge_texture_registration_v4_get_linux_dmabuf_descriptor(
    Fs25adHostBridgeTextureRegistrationV4 *texture,
    uint64_t frame_token,
    Fs25adTextureRegistrationV4LinuxDmabufDescriptor *out_descriptor);
bool fs25ad_host_bridge_texture_registration_v4_get_android_hardware_buffer_descriptor(
    Fs25adHostBridgeTextureRegistrationV4 *texture,
    uint64_t frame_token,
    Fs25adTextureRegistrationV4AndroidHardwareBufferDescriptor *out_descriptor);
/* Legacy */
bool fs25ad_host_bridge_texture_registration_v4_get_android_surface_descriptor(
    Fs25adHostBridgeTextureRegistrationV4 *texture,
    uint64_t frame_token,
    Fs25adTextureRegistrationV4AndroidSurfaceDescriptor *out_descriptor);
/* Legacy */
bool fs25ad_host_bridge_texture_registration_v4_attach_android_surface(
    Fs25adHostBridgeTextureRegistrationV4 *texture,
    const Fs25adTextureRegistrationV4AndroidSurfaceDescriptor *surface_descriptor);
/* Legacy */
bool fs25ad_host_bridge_texture_registration_v4_detach_android_surface(
    Fs25adHostBridgeTextureRegistrationV4 *texture);

/* Flutter GPU Runtime (feature: flutter-linux / flutter-android) */

Fs25adFlutterSessionHandle *fs25ad_flutter_session_new(void);
void fs25ad_flutter_session_dispose(Fs25adFlutterSessionHandle *session);
bool fs25ad_flutter_session_apply_action_json(
    const Fs25adFlutterSessionHandle *session,
    const char *action_json);
char *fs25ad_flutter_session_take_dialog_requests_json(
    const Fs25adFlutterSessionHandle *session);
bool fs25ad_flutter_session_submit_dialog_result_json(
    const Fs25adFlutterSessionHandle *session,
    const char *result_json);
bool fs25ad_flutter_session_update_overview_options_dialog_json(
    const Fs25adFlutterSessionHandle *session,
    const char *dialog_json);
char *fs25ad_flutter_session_snapshot_json(
    const Fs25adFlutterSessionHandle *session);
char *fs25ad_flutter_session_node_details_json(
    const Fs25adFlutterSessionHandle *session);
char *fs25ad_flutter_session_marker_list_json(
    const Fs25adFlutterSessionHandle *session);
char *fs25ad_flutter_session_route_tool_viewport_json(
    const Fs25adFlutterSessionHandle *session);
char *fs25ad_flutter_session_connection_pair_json(
    const Fs25adFlutterSessionHandle *session,
    uint64_t node_a,
    uint64_t node_b);
int32_t fs25ad_flutter_session_is_dirty(
    const Fs25adFlutterSessionHandle *session);
char *fs25ad_flutter_session_ui_snapshot_json(
    const Fs25adFlutterSessionHandle *session);
char *fs25ad_flutter_session_chrome_snapshot_json(
    const Fs25adFlutterSessionHandle *session);
char *fs25ad_flutter_session_dialog_snapshot_json(
    const Fs25adFlutterSessionHandle *session);
char *fs25ad_flutter_session_editing_snapshot_json(
    const Fs25adFlutterSessionHandle *session);
char *fs25ad_flutter_session_context_menu_snapshot_json(
    const Fs25adFlutterSessionHandle *session,
    int64_t focus_node_id_or_neg1);
char *fs25ad_flutter_session_viewport_overlay_json(
    const Fs25adFlutterSessionHandle *session,
    float cursor_world_x,
    float cursor_world_y);
char *fs25ad_flutter_session_viewport_geometry_json(
    const Fs25adFlutterSessionHandle *session,
    float viewport_width,
    float viewport_height);
int64_t fs25ad_flutter_session_acquire_shared_arc_raw(
    const Fs25adFlutterSessionHandle *session);
void fs25ad_flutter_session_release_shared_arc_raw(int64_t raw);

Fs25adGpuRuntimeHandle *fs25ad_gpu_runtime_new(uint32_t width, uint32_t height);
Fs25adGpuRuntimeHandle *fs25ad_gpu_runtime_new_with_session(
    const Fs25adFlutterSessionHandle *session,
    uint32_t width,
    uint32_t height);
bool fs25ad_gpu_runtime_render(Fs25adGpuRuntimeHandle *handle);
bool fs25ad_gpu_runtime_export_texture(
    Fs25adGpuRuntimeHandle *handle,
    Fs25adTextureRegistrationV4LinuxDmabufDescriptor *out_descriptor);
/* Android AHB Export (nur auf Android verfuegbar) */
bool fs25ad_gpu_runtime_export_android_hardware_buffer(
    Fs25adGpuRuntimeHandle *handle,
    Fs25adTextureRegistrationV4AndroidHardwareBufferDescriptor *out_descriptor);
bool fs25ad_gpu_runtime_resize(
    Fs25adGpuRuntimeHandle *handle,
    uint32_t width,
    uint32_t height);
void fs25ad_gpu_runtime_dispose(Fs25adGpuRuntimeHandle *handle);

#ifdef __cplusplus
}
#endif

#endif