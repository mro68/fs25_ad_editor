#ifndef FS25AD_HOST_BRIDGE_H
#define FS25AD_HOST_BRIDGE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define FS25AD_HOST_BRIDGE_ABI_VERSION 2u
#define FS25AD_HOST_BRIDGE_SHARED_TEXTURE_CONTRACT_VERSION 3u

#define FS25AD_HOST_BRIDGE_SHARED_TEXTURE_PIXEL_FORMAT_RGBA8_SRGB 1u
#define FS25AD_HOST_BRIDGE_SHARED_TEXTURE_ALPHA_MODE_PREMULTIPLIED 1u
#define FS25AD_HOST_BRIDGE_SHARED_TEXTURE_NATIVE_HANDLE_OPAQUE_RUNTIME_POINTERS 1u
#define FS25AD_HOST_BRIDGE_SHARED_TEXTURE_NATIVE_HANDLE_WGPU_POINTERS \
    FS25AD_HOST_BRIDGE_SHARED_TEXTURE_NATIVE_HANDLE_OPAQUE_RUNTIME_POINTERS

typedef struct Fs25adHostBridgeSession Fs25adHostBridgeSession;
typedef struct Fs25adHostBridgeSharedTexture Fs25adHostBridgeSharedTexture;

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

uint32_t fs25ad_host_bridge_abi_version(void);
uint32_t fs25ad_host_bridge_shared_texture_contract_version(void);
bool fs25ad_host_bridge_shared_texture_capabilities(
    Fs25adSharedTextureCapabilities *out_capabilities);

char *fs25ad_host_bridge_last_error_message(void);
void fs25ad_host_bridge_string_free(char *value);

Fs25adHostBridgeSession *fs25ad_host_bridge_session_new(void);
void fs25ad_host_bridge_session_dispose(Fs25adHostBridgeSession *session);

char *fs25ad_host_bridge_session_snapshot_json(Fs25adHostBridgeSession *session);
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

#ifdef __cplusplus
}
#endif

#endif