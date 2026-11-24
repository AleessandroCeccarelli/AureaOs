<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Desktop Rendering and Media Pipeline

## Purpose

This document records the current audit and the implementation boundary for the first usable AureaOS desktop: a wallpaper, native desktop items, mapped Wayland windows, and a working pointer.

It is an engineering decision record, not a claim that the pipeline is implemented or hardware-validated.

## Repository baseline

- Branch: `main`.
- Audited commit: `d9089aafaaa0b6470aa231f946aad1d8a7c5aede`.
- Workspace components include `aurea-desktop-api`, `aurea-compositor`, `aurea-shell`, and `aurea-desktop`.
- The compositor uses Smithay with its Wayland frontend and an exclusive Vulkan rendering path under the native DRM feature.
- No build, formatting command, test, or CI workflow was run during this audit.

## Existing implementation

### Wallpaper

The compositor already contains:

- `WallpaperAsset`: embedded JPEG bytes and bounded header metadata extraction.
- `WallpaperPixels`: owned RGBA8 data with overflow and memory-budget validation.
- `WallpaperManager` and `WallpaperSnapshot`: selection and immutable state.
- `WallpaperLayout`: Fill, Fit, Stretch and Center geometry.
- Playback policy contracts that explicitly require media decoding on a worker thread.

The missing link is a real JPEG decoder and the transfer from decoded pixels into a persistent Vulkan texture. The render path currently clears the output to the desktop background color; it does not sample a wallpaper texture.

### Vulkan composition

The existing Vulkan SHM path already provides:

- RGBA8 image allocation and upload.
- Asynchronous upload tracking and bounded surface texture caching.
- Textured-region drawing.
- Native cursor texture handling and cursor geometry.
- Frame-scoped descriptor resources and GPU completion handling.

This machinery is currently specialized for Wayland SHM surfaces and the native cursor. It is not yet a generic desktop asset pipeline. Wallpaper and desktop-item textures must have their own ownership/cache lifecycle or a deliberately generalized cache; do not force them into the Wayland surface-ID cache.

### Desktop items

The shared desktop model and `DesktopItemsSnapshot` are present and connected to `RenderScene`. The native Vulkan composition path does not yet turn those items into drawable elements. Desktop icons must remain compositor-owned desktop elements, not fake Wayland application windows.

### Pointer and input

Smithay, libinput integration, pointer state and native cursor composition already exist. The pointer is drawn after the current Wayland surfaces. Once the wallpaper is drawn as the base layer, the pointer should remain above it. Native desktop hit-testing and click behavior are separate work and must use the shared desktop geometry contract.

### Trash artwork

The six approved Trash SVGs remain the canonical source artwork in Human Interfaces. The shell currently resolves SVG bytes by theme and Trash state. The compositor has no SVG rasterization pipeline. Do not add librsvg to the core desktop solely for these icons without accepting its runtime dependency footprint.

## Linux and Debian integration findings

### JPEG decoding

Debian 13 Trixie provides libjpeg-turbo, including the TurboJPEG runtime and development packages:

- `libturbojpeg0`: runtime library.
- `libturbojpeg0-dev`: headers and development files.

The initial wallpaper is JPEG, so TurboJPEG is a narrowly scoped system codec candidate. The Rust integration should be isolated behind an Aurea-owned safe wrapper and invoked from a worker, never from the render loop. Before implementation, confirm the exact ABI/API version, Fedora development package availability, packaging requirements, error handling and allocation limits.

Do not add a general-purpose Rust image stack just to decode the one committed JPEG asset.

### SVG artwork

Debian provides librsvg, but its runtime dependency chain includes Cairo, GLib, Pango and related components. That is disproportionate for the current static icon requirement and conflicts with the goal of a lean, independent desktop stack.

Initial direction: preserve SVG as the editable canonical artwork and generate bounded raster variants as build-time/project assets using an established host tool. The installed desktop should consume those raster resources directly through Vulkan. This avoids a runtime SVG engine. The asset-generation command and reproducibility/licensing details must be documented before adding generated files.

### Linux desktop specifications

Use established FreeDesktop/XDG conventions where applicable:

- Desktop Entry for application metadata and launch.
- Icon Theme conventions for icon lookup and fallback.
- Shared MIME-info for file type identification.
- Thumbnail Managing Standard and registered thumbnailers for file thumbnails.
- Trash specification for deletion metadata and storage.
- XDG Base Directory for user data, configuration and cache locations.
- Wayland protocols and Smithay/libinput for input, surfaces and outputs.

These specifications do not provide a universal desktop-icon renderer or a complete desktop shell. AureaOS owns those product-level services and their UI.

## Service architecture

AureaOS should own coherent services, but a service does not automatically mean a permanently running process.

Initial boundaries:

- **Aurea Background**: wallpaper selection, per-output layout, lifecycle and policy. Decode work is delegated to a worker; the render thread consumes completed pixel/texture state only.
- **Aurea Media**: a narrow internal media/asset provider boundary for decoding and raster asset preparation. Start as an in-process worker/library unless measured needs justify a separate daemon or IPC.
- **Aurea Desktop Items**: desktop item model, placement, hit-testing and interaction state. It must not become a file manager or a generic Dock container.
- **Aurea Files**: filesystem operations, MIME/open-with integration, thumbnails and Trash service.
- **Aurea Shell**: Dock, launcher, panels, notifications and user-facing controls.
- **Aurea Compositor**: Wayland surfaces, outputs, pointer, hit-testing handoff and Vulkan composition/presentation. It owns GPU resources, not file discovery or image decoding policy.

Services should be lazy/on-demand where possible, bounded in memory and work, and should not duplicate Linux facilities that already meet the requirement.

## Rendering order for the first visible desktop

The intended composition order is:

1. Clear output to the selected theme fallback color.
2. Draw the wallpaper texture using the selected per-output fit mode and output-local clipping.
3. Draw native desktop items when their texture assets and geometry are ready.
4. Draw mapped Wayland surfaces in the correct stacking order.
5. Draw the native pointer/cursor above the scene.
6. Apply the existing output color pass and present through the existing DRM/KMS path.

The exact placement of the output color pass must continue to follow the existing Vulkan color pipeline contract. Do not regress Night Light or scanout synchronization while adding background textures.

## Implementation sequence

1. Add a focused architecture record and keep the development diary current.
2. Implement a small, safe TurboJPEG adapter behind an Aurea-owned media boundary; enforce input/output size limits and decode off-thread.
3. Add a bounded wallpaper texture resource/cache using the existing Vulkan image/upload primitives. Do not reuse the Wayland surface cache by inventing fake surface IDs.
4. Draw the wallpaper before windows and cursor, using the existing fit/layout contract.
5. Produce raster variants of the approved static Trash artwork and add a dedicated desktop asset texture path.
6. Convert desktop item snapshots into render elements, preserving logical-to-output coordinate conversion and clipping.
7. Connect pointer hit-testing and click behavior to desktop items only after visible rendering is working.
8. Validate on Fedora hardware only after the user agrees to the first build/test milestone.

## Explicit non-goals for this milestone

- No ISO generation.
- No build, Cargo check, formatting, tests or CI until the agreed visual milestone and user approval.
- No X11, XWayland or OpenGL.
- No GNOME/KDE desktop dependencies.
- No permanent background daemon unless profiling or lifecycle requirements demonstrate a real need.
- No copying COSMIC implementation code or assets.
- No replacing the existing Vulkan pipeline or restructuring unrelated modules.

## Audit limitations

This audit inspected the repository source at the recorded commit and Debian package metadata. It did not run the code, inspect the running Fedora host, prove the presence of runtime libraries on that host, or validate physical DRM/KMS presentation. TurboJPEG remains a selected candidate pending its packaging and ABI integration review.
