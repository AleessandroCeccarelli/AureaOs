# AureaOS Wallpaper System

Status: architecture decision / implementation plan
Date: 2026-11-23
License: GPL-3.0-or-later

## Product goal

AureaOS provides a first-class **static-image wallpaper service** with the everyday selection and layout capabilities users expect from Windows and macOS. The already selected `aureaos-midnight-flow.jpg` remains the default AureaOS wallpaper.

This scope handles still raster images only. It does not include video wallpapers, animated-image playback, audio, GStreamer, FFmpeg playback pipelines, PipeWire media routing, or a media-player subsystem. A future time-of-day wallpaper may switch between separate still images; it must not rely on continuous video playback.

The wallpaper service is independent from the Vulkan renderer. The renderer receives validated RGBA8 pixels and per-output placement data; it does not decode image formats.

## Still-image format policy

The authoritative closed list is maintained in [`../architecture/MEDIA-FORMATS.md`](../architecture/MEDIA-FORMATS.md), with an Italian companion at [`../architecture/MEDIA-FORMATS-it.md`](../it/FORMATS-MULTIMEDIALI.md).

AureaOS accepts only the following user-selected raster formats:

| Format | Extensions | Policy |
|---|---|---|
| JPEG | `.jpg`, `.jpeg`, `.jpe` | Required |
| PNG | `.png` | Required |
| BMP / DIB | `.bmp`, `.dib` | Required |
| GIF | `.gif` | Required as a static image; decode the first frame only |
| TIFF | `.tif`, `.tiff` | Required; decode the first page only |
| WebP | `.webp` | Required |
| HEIF / HEIC | `.heif`, `.heic` | Required; still-image decode, including HEVC-coded photos |
| AVIF | `.avif` | Required; still-image decode |
| JPEG 2000 | `.jp2`, `.j2k`, `.jpf`, `.jpx` | Conditional on Debian-compatible decoder, license and footprint review |
| OpenEXR | `.exr` | Deferred pending HDR/color-management policy |

GIF, APNG, animated WebP, video, PDF, SVG, camera RAW, JPEG 2000 and OpenEXR are not accepted wallpaper inputs under registry v1.0.

Windows theme documentation lists BMP, GIF, JPEG, PNG and TIFF wallpaper assets; its theme-pack list also includes DIB. Some managed Windows configuration interfaces intentionally narrow the list to JPEG/PNG/BMP. Apple's wallpaper settings let users select an image file, a photo or a photo set; Apple's broader image tooling supports additional still-image formats. AureaOS adopts a practical superset rather than treating one narrow management interface as the full desktop capability.

Extensions are chooser hints only. The importer must identify formats from file signatures/container metadata and validate the actual decoded content.
## Layout modes

Per output, support Fill, Fit, Stretch, Center, Tile, and Span.

- Fill: preserve aspect ratio and crop overflow.
- Fit: preserve aspect ratio and leave margins.
- Stretch: scale independently on each axis.
- Center: native pixel size, centered and clipped to the output.
- Tile: repeat a source image over the output.
- Span: place one desktop-wide composition across the selected output group.

Each output uses its own physical dimensions, scale, and local coordinate space. Span must use the combined logical arrangement and account for gaps, mixed resolutions, and output transforms.

## Static image collections

The service may later support a collection of still images that changes on a user-configured schedule, including a day/night sequence. Every item remains a normal still image decoded through the same bounded image pipeline. There is no animated-image playback, video decoding, or audio.

The service should support per-monitor selection, a shared wallpaper across monitors, and a single panoramic composition spanning a monitor group.

## Proposed architecture boundaries

- `WallpaperManager`: selection, per-output assignment and fit mode.
- Still-image decoder modules: format-specific decode into `WallpaperPixels`.
- Optional still-image collection scheduler: changes the selected asset at configured intervals; it does not decode or play video.
- `WallpaperFrameCache`: bounded CPU/GPU image cache with explicit ownership and invalidation.
- Vulkan wallpaper renderer: uploads ready pixels and draws them below application surfaces and above the clear color.
- Power/session policy: pauses collection scheduling when the session is locked, suspended or in power-saving mode.

Do not bind the wallpaper cache to Wayland surface ObjectIds. Wallpaper assets have their own identity and lifecycle. Keep pending GPU resources alive until their submissions complete.
## Decoder and dependency policy

- Keep `aurea-media` as the sole public image-decoding API and keep codec-specific code behind private decoder adapters.
- Every supported decoder returns the same owned, tightly packed RGBA8 buffer with checked width, height, stride and exact byte length.
- Preserve explicit input, axis, pixel-count and output-memory limits. Enforce dimensions and allocation limits before pixel allocation; do not rely only on post-decode validation.
- Decode only on the bounded media worker, never on a compositor frame/render callback.
- Reject malformed/truncated files, unsupported bit depth/color spaces, oversized dimensions, excessive frame/page counts and allocation failures with stable service errors.
- GIF is not an accepted wallpaper format. Multi-page TIFF is treated as a still image by selecting its first page.
- HEIC/HEIF uses libheif with the HEVC still-image decoder plugin. This is image decoding, not HEVC video playback.
- AVIF uses libheif with a suitable AV1 still-image decoder plugin such as dav1d.
- Do not add codec dependencies to the compositor. Native libraries belong to `aurea-media`; only decoded RGBA8 crosses the boundary.
- Install only required decoder plugins. Do not use broad plugin metapackages or add encoders when AureaOS only needs decoding.
- Debian 13 Trixie is the dependency authority; Fedora is the development/validation environment.

## Implementation sequence

1. Complete and validate the JPEG path: decode, bounded RGBA8, dedicated Vulkan upload/cache, and draw pass for empty and populated scenes.
2. Add and validate PNG, BMP, TIFF and WebP decoders.
3. Complete owned user-selected assets and content-signature detection.
4. Integrate HEIC/HEIF and AVIF through a Debian-compatible libheif backend and only the required decoder plugins.
5. Complete Tile and Span layout, including multi-output geometry tests.
6. Add an optional still-image collection scheduler and power/session policy.
7. Add the UI for selecting static images, per-monitor assignments and still-image collections.

## Current implementation status (2026-11-24)

Implemented in code but not yet verified on Fedora:
- embedded selected JPEG asset metadata;
- WallpaperManager and immutable snapshot path;
- Fill, Fit, Stretch, Center layout calculations;
- bounded owned RGBA8 pixel buffer;
- `VulkanWallpaperTexture` resource: sampled RGBA8 image allocation, staging upload and explicit shader-read transition; asynchronous `submit_pixels` returns a fence-backed pending upload, while `from_pixels` remains a blocking convenience wrapper.
- Bounded per-device `VulkanWallpaperTextureCache`: asset-ID/revision key, one ready selection, at most two pending uploads and a 256 MiB pending-byte budget; frame preparation polls fences without blocking. Cache population and scene drawing are still pending.

The output-local draw geometry is now implemented as a pure CPU contract: it clips destination rectangles to each output and computes the matching normalized UV rectangle, including Fill crop and Center clipping. This is not yet a Vulkan draw pass and does not display the wallpaper. The geometry contract is exported from the desktop API and has unit tests for Fill, Fit, Stretch, Center, and non-intersection. These tests have not been run locally.

Not yet implemented:
- JPEG pixel decoding and display;
- wallpaper identity cache and draw pass below desktop surfaces;
- PNG/BMP decoders;
- Tile and Span;
- animated image/video playback (excluded by product decision);
- dynamic time-of-day collections and UI.

The Vulkan wallpaper texture upload, asynchronous submission path and cache lifecycle are not yet verified by Cargo build/tests or hardware. No Cargo build, format, test, or hardware validation is claimed by this document.


Implemented since the CPU geometry milestone:
- Added dedicated `wallpaper_texture.vert` and `.frag` shaders. The vertex shader maps a caller-provided normalized UV rectangle; the fragment shader samples the wallpaper descriptor.
- `build.rs` compiles the wallpaper shaders only for the DRM/Vulkan build and exports their SPIR-V paths.
- `VulkanSurfacePipelineCache` now owns a separate wallpaper pipeline per target format. Its vertex push-constant range is a four-float UV rectangle, independent of the existing surface transform ABI.

This is pipeline preparation only: no descriptor binding or wallpaper draw is issued yet, and the decoder/cache still do not provide a ready selected texture. Local Fedora validation has not been run.

The Vulkan textured draw-pass API now also accepts caller-defined push-constant payloads.

### Frame-composition integration

The Vulkan SHM compositor now consumes a wallpaper texture only when the wallpaper is selected in the immutable desktop snapshot and its upload has completed. It creates a frame-scoped sampled-image descriptor, records the dedicated wallpaper pipeline immediately after the clear pass, and then draws application surfaces and the cursor above it. The wallpaper-only/empty-scene path no longer exits through the clear-only fast path when a ready selected texture exists. Clearing the desktop wallpaper selection also clears the cached texture.

The current integration uses `WallpaperFitMode::Fill` and output-local physical dimensions.

### Asset identity safety

The frame compositor now resolves the selected asset ID from `DesktopSnapshot` and requests a ready GPU texture only when the cache's selected key has that same asset ID. This prevents a previous wallpaper texture from being drawn after the user changes selection but before the replacement upload completes. The cache still tracks revisions independently; the current immutable embedded asset identity is matched by ID at the frame boundary. It is ready to draw a populated `VulkanWallpaperTextureCache`, but the JPEG decoder and the producer that calls `prepare_wallpaper_texture` are still absent. Therefore the embedded JPEG is not yet displayed; no user-visible wallpaper is claimed.

`WallpaperUvRect::to_push_constant_bytes` now serializes the normalized UV rectangle in the exact `vec4(min.x, min.y, max.x, max.y)` order expected by the dedicated wallpaper vertex shader. A unit test checks the byte round-trip. This remains CPU-side preparation; no descriptor binding or wallpaper draw is wired into the frame yet. It validates that payloads are non-empty, four-byte aligned, and no larger than Vulkan's guaranteed 128-byte minimum. The existing surface path still supplies its original four-byte transform; the wallpaper pipeline can supply its independent 16-byte UV rectangle. This API layer does not yet bind or draw the wallpaper.


## Product clarification — GIF excluded, TIFF retained (2026-11-24)

GIF is not a wallpaper format in AureaOS. The importer rejects GIF signatures; there is no GIF decoder feature in `aurea-media`, no GIF entry in `WallpaperFormat`, and no animated-frame provider or playback policy in the wallpaper manager. This is an explicit product boundary, not merely a decision to display the first GIF frame.

TIFF remains supported as a still-image format. It is commonly used for high-quality photography, scanning, publishing and archival workflows. It can be lossless and support high bit depth, but files can be large. A multi-page TIFF is treated as a static wallpaper source using its first page; no page animation or slideshow behavior is implied.
