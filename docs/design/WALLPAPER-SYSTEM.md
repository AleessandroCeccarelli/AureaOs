# AureaOS Wallpaper System

Status: architecture decision / implementation plan
Date: 2026-11-23
License: GPL-3.0-or-later

## Product goal

AureaOS provides a first-class wallpaper service comparable in everyday capability to the desktop wallpaper experience users expect from Windows and macOS. The already selected `aureaos-midnight-flow.jpg` remains the default AureaOS wallpaper. Supporting more formats and animated wallpapers must not change that design decision.

The wallpaper service is independent from the Vulkan renderer. The renderer receives ready-to-display frames and per-output placement data; it does not decode image or video formats.

## Still-image support

Planned format tiers:

- Initial: JPEG, PNG, BMP.
- Later: WebP and TIFF, subject to maintained decoder availability and licensing review.
- GIF: support as an animated image format where practical, but do not make GIF the preferred format for high-resolution full-screen motion.
- HEIF/HEIC: evaluate separately; do not promise support until a maintained decoder and distribution story are selected.

All still-image decoders normalize to the bounded `WallpaperPixels` RGBA8 contract. Decoding must validate dimensions, arithmetic, byte length, and an explicit memory budget before allocation.

## Layout modes

Per output, support Fill, Fit, Stretch, Center, Tile, and Span. The existing `WallpaperLayout` work covers the first four modes. Tile and Span remain planned.

- Fill: preserve aspect ratio and crop overflow.
- Fit: preserve aspect ratio and leave margins.
- Stretch: scale independently on each axis.
- Center: native pixel size, centered and clipped to the output.
- Tile: repeat a source image over the output.
- Span: place one desktop-wide composition across the selected output group.

Each output uses its own physical dimensions, scale, and local coordinate space. Span must use the combined logical arrangement and account for gaps, mixed resolutions, and output transforms.

## Animated wallpaper capabilities

Two distinct animated experiences are planned:

1. **Looping motion wallpaper**: a video or animated image plays continuously, optionally with a configured loop and muted audio by default.
2. **Dynamic wallpaper collection**: a set of still images or scenes changes according to a schedule, such as time of day. This can reproduce the day/night progression users associate with dynamic desktop wallpapers without requiring continuous video decoding.

The service should also support per-monitor selection, shared wallpaper across monitors, and a single panoramic composition spanning a monitor group.

### Playback and power policy

- Pause decoding and frame production when the wallpaper is fully obscured, the output is disabled, the session is locked, or the system is suspended.
- Resume from a defined playback position or restart according to the user's selected policy.
- Use output refresh and frame timestamps for presentation; do not decode a new frame on every compositor repaint.
- Keep audio disabled by default. Wallpaper playback must never unexpectedly produce sound.
- Provide a static fallback frame when decoding fails or resources are unavailable.
- Respect battery/power-saving mode and expose a reduced-motion/static-wallpaper preference.
- Bound decoded frame dimensions, frame rate, queued frames, and memory use.
- Do not block the compositor's render thread while decoding or reading media.

## Proposed architecture boundaries

- `WallpaperManager`: selection, per-output assignment, fit mode, slideshow and dynamic schedule.
- Still-image decoder modules: format-specific decode into `WallpaperPixels`.
- Animated media source: isolated decoder backend that yields timestamped RGBA frames; backend selection remains open pending dependency, licensing, packaging, and Fedora validation review.
- `WallpaperFrameCache`: bounded CPU/GPU frame cache with explicit ownership and invalidation.
- Vulkan wallpaper renderer: uploads ready frames and draws them below application surfaces and above the clear color.
- Power/session policy: tells playback when to pause, resume, or switch to a static frame.

Do not bind the wallpaper cache to Wayland surface ObjectIds. Wallpaper assets have their own identity and lifecycle. Keep pending GPU resources alive until their submissions complete.

## Media backend decision

No video-decoding backend is selected yet. Evaluate maintained options (for example GStreamer integration or a focused FFmpeg-based backend) against:
- Fedora and target-distribution packaging;
- codec availability and patent/licensing implications;
- Rust API maturity and maintenance;
- hardware decode availability and safe software fallback;
- memory limits, timestamp handling, and zero-copy feasibility;
- compatibility with the project's Wayland-only/Vulkan-only presentation path.

Do not add a media dependency or edit Cargo.lock speculatively. The compositor must remain responsive if the decoder stalls or fails.

## Implementation sequence

1. Complete and validate the static JPEG path: decode, bounded RGBA8, dedicated Vulkan upload/cache, draw pass for empty and populated scenes.
2. Add PNG and BMP decoders using maintained libraries and Cargo-generated lockfile.
3. Complete Tile and Span layout, including multi-output geometry tests.
4. Define the animated-frame source interface and deterministic playback clock/state tests.
5. Select and integrate a media decoder backend after dependency, packaging, and license review.
6. Add pause/resume policy, frame queue limits, fallback frame, reduced-motion and power-saving behavior.
7. Add UI for choosing static, slideshow, dynamic day/night, and looping motion wallpapers.

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
- animated image/video playback;
- dynamic time-of-day collections and UI.

The Vulkan wallpaper texture upload, asynchronous submission path and cache lifecycle are not yet verified by Cargo build/tests or hardware. No Cargo build, format, test, or hardware validation is claimed by this document.
