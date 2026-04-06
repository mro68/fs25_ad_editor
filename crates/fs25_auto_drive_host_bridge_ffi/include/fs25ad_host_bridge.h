#ifndef FS25AD_HOST_BRIDGE_H
#define FS25AD_HOST_BRIDGE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define FS25AD_HOST_BRIDGE_ABI_VERSION 1u
#define FS25AD_HOST_BRIDGE_CANVAS_CONTRACT_VERSION 1u

#define FS25AD_HOST_BRIDGE_CANVAS_PIXEL_FORMAT_RGBA8_SRGB 1u
#define FS25AD_HOST_BRIDGE_CANVAS_ALPHA_MODE_PREMULTIPLIED 1u

typedef struct Fs25adHostBridgeSession Fs25adHostBridgeSession;
typedef struct Fs25adHostBridgeNativeCanvas Fs25adHostBridgeNativeCanvas;

typedef struct Fs25adRgbaFrameInfo {
    uint32_t width;
    uint32_t height;
    uint32_t bytes_per_row;
    uint32_t pixel_format;
    uint32_t alpha_mode;
    size_t byte_len;
} Fs25adRgbaFrameInfo;

uint32_t fs25ad_host_bridge_abi_version(void);
uint32_t fs25ad_host_bridge_canvas_contract_version(void);

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

Fs25adHostBridgeNativeCanvas *fs25ad_host_bridge_canvas_new(uint32_t width, uint32_t height);
void fs25ad_host_bridge_canvas_dispose(Fs25adHostBridgeNativeCanvas *canvas);
bool fs25ad_host_bridge_canvas_resize(
    Fs25adHostBridgeNativeCanvas *canvas,
    uint32_t width,
    uint32_t height);
bool fs25ad_host_bridge_canvas_render_rgba(
    Fs25adHostBridgeSession *session,
    Fs25adHostBridgeNativeCanvas *canvas);
bool fs25ad_host_bridge_canvas_last_frame_info(
    const Fs25adHostBridgeNativeCanvas *canvas,
    Fs25adRgbaFrameInfo *out_info);
bool fs25ad_host_bridge_canvas_copy_last_frame_rgba(
    const Fs25adHostBridgeNativeCanvas *canvas,
    uint8_t *dst,
    size_t dst_len);

#ifdef __cplusplus
}
#endif

#endif