# API der Render-wgpu-Crate

## Ueberblick

`fs25_auto_drive_render_wgpu` enthaelt den host-neutralen wgpu-Renderer-Kern. Die Crate konsumiert ausschliesslich read-only Render-Snapshots (`RenderScene` + `RenderAssetsSnapshot`) und kennt weder `egui`, `eframe` noch Flutter-spezifische SDK-Typen.

Seit dem Shared-Texture-Hard-Cut ist der alte RGBA-Pixelbuffer-Pfad entfernt. Offscreen-Hosts nutzen jetzt ausschliesslich `SharedTextureRuntime` mit explizitem Acquire/Release-Lifecycle. Die ABI-Versionierung des opaque Runtime-Vertrags sitzt bewusst im FFI-Adapter; der egui-Onscreen-Host bleibt ein direkter `RenderPass`-Pfad ueber denselben RenderFrame-Seam.

Additiv dazu exponiert die Crate jetzt einen separaten `Texture-Registration-v4`-Vertrag fuer kommende Host-Texture-Registration auf Windows, Linux und Android. `v4` laeuft parallel zu `v3`, nutzt gemeinsame Capability-Negotiation und gemeinsame Frame-Metadaten, trennt aber plattformspezifische Payload-Familien explizit.

Der v4-Vertrag ist bewusst nur der additive Transport- und Lifecycle-Slice. Echte externe Host-Registration braucht weiterhin zusaetzliche native Host-Pfade ausserhalb dieses Render-Core, damit Windows-, Linux- oder Android-Consumer die jeweilige Payload-Familie auch wirklich importieren oder attachen koennen.

## Kompatibilitaet (Stand: 2026-04-06)

- Rust-Edition: `2024`
- GPU-Backend: `wgpu 29.0.*`
- Pipeline-Layouts nutzen die aktuellen `wgpu`-29-Deskriptoren (`bind_group_layouts` mit `Option`, `immediate_size`, `multiview_mask`).

## Komponenten

| Komponente | Verantwortung |
|---|---|
| `lib.rs` | Oeffentliche Root-API (`Renderer`, `RendererTargetConfig`, Shared-Texture-Typen) |
| `export_core.rs` | Interner, transportneutraler Export-Kern (Target-Guards, Background-Sync, Offscreen-Renderpass) |
| `shared_texture.rs` | Shared-Texture-Runtime mit Frame-Lifecycle und opaque Runtime-Handle-Metadaten |
| `texture_registration/*` | Additiver `v4`-Vertrag (Capabilities, Lifecycle-State-Machine, plattformspezifische Payload-Familien) |
| `background_renderer.rs` | Hintergrund-Quad, Upload und zoomabhaengiges Sampling |
| `marker_renderer.rs` | Marker-Instancing und Pin-Texturpfad |
| `connection_renderer/` | Linien, Pfeile und Viewport-Culling fuer Verbindungen |
| `node_renderer.rs` | Node-Instancing und Selektion-Rendering |
| `texture.rs` | Texture-/Sampler-Erstellung aus `DynamicImage` |

## Oeffentliche Typen

| Typ | Zweck |
|---|---|
| `Renderer` | Host-neutraler GPU-Renderer fuer `RenderScene` |
| `RendererTargetConfig` | Zielkonfiguration des Render-Targets (`color_format`, `sample_count`) |
| `SharedTextureRuntime` | Offscreen-Shared-Texture-Runtime ohne CPU-Readback |
| `SharedTextureFrame` | Metadaten eines gerenderten/geleasten Shared-Texture-Frames |
| `SharedTextureNativeHandle` | Opaque Runtime-Pointerwerte (`texture_ptr`, `texture_view_ptr`) fuer denselben Prozessraum |
| `SharedTexturePixelFormat` | Aktuell fest verdrahtet: `Rgba8Srgb` |
| `SharedTextureAlphaMode` | Aktuell fest verdrahtet: `Premultiplied` |
| `SharedTextureError` | Fehler fuer Groesse, Viewport-Mismatch und Frame-Lease-Lifecycle |
| `TextureRegistrationCapabilities` | Gemeinsame v4-Capabilities inkl. Plattformzeilen fuer Windows/Linux/Android |
| `TextureRegistrationPlatformCapabilities` | Plattformspezifische v4-Zeile (`platform`, `model`, `payload_family`, `availability`) |
| `TextureRegistrationLifecycle` | Modellunabhaengige v4-State-Machine fuer Render/Acquire/Release/Resize sowie Android Attach/Detach |
| `TextureRegistrationLifecycleState` | Beobachtbarer Lifecycle-Zustand eines v4-Pfads |
| `TextureRegistrationLifecycleError` | Fehler fuer Lease-, Resize- und Android-Attach-Guards im v4-Lifecycle |
| `TextureRegistrationFrameMetadata` | Gemeinsame v4-Frame-Metadaten (`width`, `height`, `texture_id`, `texture_generation`, `frame_token`) |
| `WindowsDescriptorKind` | Untertyp der Windows-Descriptorfamilie (`DxgiSharedHandle` oder `D3d11Texture2D`) |
| `WindowsDescriptor` | Windows-Descriptorfamilie (`DxgiSharedHandle` oder `D3d11Texture2D`) |
| `LinuxDmabufPlane` | Einzelne DMA-BUF-Plane des Linux-v4-Vertrags |
| `LinuxDmabufDescriptor` | Linux-DMA-BUF-Descriptorfamilie mit Plane-Liste |
| `AndroidAttachmentKind` | Untertyp des Android-Host-Attach-Modells |
| `AndroidSurfaceDescriptor` | Android-Surface-Attachment-Descriptorfamilie |
| `BackgroundWorldBounds` | Weltkoordinaten des Background-Quads im 2D-Koordinatensystem des Render-Core (`x/y`) |
| `RenderScene` | Re-exportierter per-frame Render-Vertrag aus `fs25_auto_drive_engine::shared` |
| `RenderQuality` | Re-exportierte Qualitaetsstufe des Render-Vertrags |

## Oeffentliche Re-Exports

- `pub use fs25_auto_drive_engine::shared;` - Zugriff auf den stabilen Snapshot-Vertrag aus derselben Crate-Oberflaeche

## Oeffentliche Methoden

| Signatur | Zweck |
|---|---|
| `Renderer::new(device, queue, target_config)` | Erstellt den Renderer mit raw `wgpu` und initialisiert alle Sub-Renderer |
| `Renderer::render_scene(device, queue, render_pass, scene)` | Rendert den aktuellen `RenderScene`-Snapshot |
| `Renderer::set_background(device, queue, image, world_bounds, scale)` | Setzt oder aktualisiert das Background-Asset im Kern |
| `Renderer::clear_background()` | Entfernt das Background-Asset |
| `SharedTextureRuntime::new(device, queue, size)` | Erstellt eine Offscreen-Shared-Texture-Runtime |
| `SharedTextureRuntime::resize(device, size)` | Realloziert das Offscreen-Ziel bei Groessenaenderung |
| `SharedTextureRuntime::render_frame(device, queue, scene, assets)` | Synchronisiert Assets revisionsbasiert und rendert den Frame in die Shared-Texture |
| `SharedTextureRuntime::acquire_frame()` | Leased den zuletzt gerenderten Frame fuer den Host |
| `SharedTextureRuntime::release_frame(frame_token)` | Gibt den aktiven Frame-Lease wieder frei |
| `SharedTextureRuntime::frame()` | Liefert die Metadaten des zuletzt gerenderten Frames ohne Lease-Aenderung |
| `SharedTextureRuntime::native_handle(frame_token)` | Liefert opaque Runtime-Pointerwerte fuer den aktiven Lease |
| `query_texture_registration_v4_capabilities()` | Liefert die additive v4-Capability-Matrix fuer Windows/Linux/Android |
| `TextureRegistrationLifecycle::record_render(...)` | Registriert einen neuen v4-Frame in der Lifecycle-State-Machine |
| `TextureRegistrationLifecycle::acquire_frame()` | Leased den zuletzt registrierten v4-Frame |
| `TextureRegistrationLifecycle::release_frame(frame_token)` | Gibt den aktiven v4-Lease wieder frei |
| `TextureRegistrationLifecycle::on_resize()` | Invalidiert den letzten v4-Frame nach Resize/Recreate |
| `TextureRegistrationLifecycle::attach_surface()` / `detach_surface()` | Android-spezifische Attach/Detach-Guards im Host-attached-Modell |

## Shared-Texture-Vertrag

- Offscreen-Farbformat: `wgpu::TextureFormat::Rgba8UnormSrgb`
- Sample-Count: `1`
- Clear-Farbe: transparentes Schwarz
- Exportierter Alpha-Modus: `Premultiplied`
- Exportiertes Pixel-Format: `RGBA8 sRGB`
- Zielgroessen werden gegen `0` und `max_texture_dimension_2d` validiert.
- Background-Sync ist revisionsbasiert ueber `RenderAssetsSnapshot::background_asset_revision()` und `background_transform_revision()`.
- Acquire/Release ist explizit: Solange ein Frame geleast ist, blockiert die Runtime `render_frame()` und `resize()` mit `SharedTextureError::FrameInUse`.
- `SharedTextureNativeHandle` enthaelt opaque Runtime-Pointerwerte fuer denselben Prozessraum, keine backend-nativen Vulkan-/Metal-/DX-Interop-Handles.
- Die Versionierung dieses opaque Runtime-Vertrags liegt bewusst im FFI-Adapter (`FS25AD_HOST_BRIDGE_SHARED_TEXTURE_CONTRACT_VERSION = 3`).
- `SharedTextureRuntime` ist der einzige Offscreen-Transportpfad im Rust-Repo; ein Pixelbuffer-Fallback existiert nicht mehr.
- Domain-X/Z wird intern auf Render-X/Y umgelegt (`min_z/max_z -> min_y/max_y`).

## Additiver Texture-Registration-v4-Vertrag

- `v3` bleibt unveraendert ein opaque same-process Runtime-Vertrag.
- `v4` ist rein additiv und hat eine eigene Vertragsversion (`TEXTURE_REGISTRATION_V4_CONTRACT_VERSION = 4`).
- Gemeinsamer Kern in `v4`:
	- Capability-Negotiation (`query_texture_registration_v4_capabilities`)
	- gemeinsame Frame-Metadaten (`TextureRegistrationFrameMetadata`)
	- gemeinsame Lifecycle-State-Machine (`TextureRegistrationLifecycle`)
- Plattformspezifische Payload-Familien sind getrennt modelliert:
	- Windows: `WindowsDescriptor`
	- Linux: `LinuxDmabufDescriptor`
	- Android: `AndroidSurfaceDescriptor`
- Der Vertrag allein macht noch keinen produktiven externen Host-Interop. Dafuer braucht es pro Plattform zusaetzlich native Host-Pfade ausserhalb dieses Render-Core, etwa fuer DXGI-/D3D11-Import, DMA-BUF-Import oder Android-Surface-Lifecycle.
- Stand dieser Ausbaustufe: Die Plattformpfade sind als `NotYetImplemented` bzw. `Unsupported` explizit capability-gated; es gibt keinen stillen Rueckfall auf Pixelbuffer oder v3-Pointer-Reinterpretation.
- Konkreter Backend-Blocker:
	- Windows: Der aktuelle Offscreen-Pfad erzeugt regulaere `wgpu::Texture`-Objekte ueber `Device::create_texture`; `wgpu 29` hat dort keine Export-/Shared-Handle-Felder. Der unsichere `as_hal`-Abstieg liefert zwar ein internes `ID3D12Resource`, erzeugt aber weder einen produktiven DXGI-Shared-Handle- oder `ID3D11Texture2D`-Registrationspfad noch den dazugehoerigen nativen Host-Importpfad.
	- Linux: Derselbe Offscreen-Pfad allokiert keine exportierbare Vulkan-External-Memory; im Repo existiert daher weder DMA-BUF-FD-/Modifier-Export noch ein nativer DMA-BUF-Importpfad fuer diese Descriptoren.
	- Android: Der Renderer rendert aktuell nur in eine interne Offscreen-Textur. Fuer produktiven v4-Surface-Attach braucht es stattdessen ein hostseitig erzeugtes `ANativeWindow`-/Surface-Ziel, backend-spezifisches Rendern gegen dieses Ziel und einen nativen Host-Surface-Lifecycle.

## Beispiel

```rust
let target_config = RendererTargetConfig::new(surface_format, 4);
let mut renderer = Renderer::new(device, queue, target_config);
renderer.render_scene(device, queue, render_pass, &scene);

let mut runtime = SharedTextureRuntime::new(device, queue, [800, 600])?;
runtime.render_frame(device, queue, &scene, &assets)?;
let frame = runtime.acquire_frame()?;
let native = runtime.native_handle(frame.frame_token)?;
runtime.release_frame(frame.frame_token)?;
assert!(native.texture_ptr > 0);
```

## Datenfluss

```mermaid
flowchart LR
	HOST[Host-Adapter] --> FRAME[RenderScene + RenderAssetsSnapshot]
	FRAME --> RUNTIME[SharedTextureRuntime]
	RUNTIME --> CORE[RenderExportCore]
	CORE --> RENDERER[Renderer]
	RENDERER --> PASS[Offscreen RenderPass]
	RUNTIME --> META[SharedTextureFrame]
	RUNTIME --> HANDLE[SharedTextureNativeHandle]
	META --> HOST
	HANDLE --> HOST
```

## Scope

- Diese Crate enthaelt nur den GPU-Kern und keine Host-Callback-Logik.
- Host-spezifische Adapter (egui-Callback, C-ABI, Flutter-Glue) bleiben in den Host-Crates.
- `SharedTextureRuntime` ersetzt den frueheren Pixelbuffer-Pfad vollstaendig.
- Der egui-Onscreen-Pfad bleibt bewusst ein direkter `RenderScene`-Paint-Callback ueber `egui_wgpu` und wird nicht als Shared-Texture-Transport beschrieben.
