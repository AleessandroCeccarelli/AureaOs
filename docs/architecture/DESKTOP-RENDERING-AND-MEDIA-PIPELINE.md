<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Desktop Rendering and Media Pipeline

## Purpose

This document records the implementation state and boundaries for the first usable AureaOS desktop: wallpaper, native desktop items, mapped Wayland windows, and pointer interaction. It distinguishes code present in the repository from behavior that has been validated on physical hardware.

## Platform and engineering constraints

- Debian 13 Trixie is the target system; Fedora is the development host only.
- The compositor uses Smithay, Wayland and Vulkan. X11, XWayland and OpenGL are excluded.
- Keep modules small and single-purpose. The compositor owns GPU resources and presentation, not file discovery or media policy.
- Decode and asset preparation must stay off the render thread. Queues and memory budgets must be bounded.
- New native packages require Debian availability, licensing, build-time/runtime classification and least-privilege review before adoption.
- Do not add a permanent daemon where an in-process bounded worker is sufficient.

## Current implementation

### Desktop appearance and wallpaper

Implemented in the compositor:

- `DesktopManager`, Light/Dark theme tokens, fallback background color and immutable per-frame `DesktopSnapshot`.
- `WallpaperManager`, embedded `WallpaperAsset`, bounded JPEG header/dimension validation, `WallpaperPixels` RGBA8 validation and Fill/Fit/Stretch/Center geometry.
- `aurea-media`, a separate workspace crate with a private TurboJPEG C shim and Rust-facing decode API. JPEG decoding runs on a named worker thread with a one-request waiting queue and explicit compressed-input, dimensions, pixel-count and output-memory limits.
- `WallpaperDecodeService` tracks selection revisions, polls without blocking, discards stale results and distinguishes temporary queue saturation from decode failure.
- A dedicated bounded Vulkan wallpaper texture cache owns pending uploads until their fences complete. A texture is exposed only when its asset/revision is still selected.
- The Vulkan frame path polls decode/upload work without waiting, draws the wallpaper after the fallback clear and before Wayland surfaces, and retains the native cursor above the scene. Removing the selected wallpaper clears its visible cached texture.
- Wallpaper fit geometry is calculated per output in physical target coordinates; the wallpaper uses its own shader/push-constant ABI and does not reuse fake Wayland surface IDs.

**Validation boundary:** source integration is present, but no successful Fedora hardware display test is recorded. The TurboJPEG ABI probe passed in Debian 13 and Fedora GitHub Actions containers; that validates the exercised native ABI path, not the complete Rust production integration. The current Rust integration and Vulkan presentation still require the agreed grouped verification on Fedora.

### Vulkan composition and failure handling

The existing renderer retains separate ownership for Wayland SHM textures, wallpaper textures and the native cursor. Uploads are fence-backed and asynchronous where implemented. Descriptor resources remain frame-scoped until GPU completion is established.

Vulkan `ERROR_DEVICE_LOST` is propagated as a fatal typed condition through the reviewed wallpaper, SHM, cursor-upload and scanout paths. The current policy is fail-stop: do not attempt a CPU fallback that still depends on the lost logical device, and do not retire GPU/KMS resources while completion is uncertain. Full device recreation is not implemented.

### Native desktop items

The technology-independent desktop API already defines:

- Stable `DesktopItemId`, item kinds (File, Folder, Application, Trash) and logical positions.
- Desktop preferences and visual grouping.
- `DesktopRect` and deterministic `DesktopHitTest`, including overlap ordering and logical-coordinate semantics.
- A compositor-owned `DesktopItemsSnapshot` captured into `RenderScene`.

The native Vulkan composition path does **not** yet draw desktop item icons. A first per-output geometry module now converts each item's global logical position into physical output geometry, applying integer/fractional scale, output transform and clipping. It uses a 64-logical-pixel icon destination and skips invalid/off-output entries independently. This module is staged but not yet connected to the Vulkan draw path. The hit-test contract exists, but desktop item hit testing is not yet connected to physical pointer events. Do not treat unit tests as proof of end-to-end icon interaction.

### Pointer, output hotplug and session lifecycle

- Relative motion, buttons, axes/scroll and touchpad gestures arrive through libinput/Smithay.
- Pointer coordinates are global logical coordinates and are constrained to the union of enabled output rectangles. Gaps resolve to the nearest output edge.
- Primary-button presses already focus/raise a Wayland toplevel and are forwarded through the Wayland pointer.
- Native cursor geometry is resolved per output using logical position, cursor scale, target scale/fractional scale and output transform.
- Output reconciliation refreshes pointer focus and requests frames. DRM-device removal unregisters its outputs and attempts to place the pointer on the primary remaining output.
- On libseat pause, input events are ignored; KMS surfaces are cleared before graphics resources are released. On activation, GBM/Vulkan/KMS and deferred outputs are restored before the pointer is centered on an enabled output.

**Known limitation:** centering is deliberately skipped while a client owns an active pointer grab. The pointer is constrained on subsequent physical motion, but the end-of-grab transition after removal of the output under the pointer still deserves a focused lifecycle review. No hardware hotplug/session-switch validation is recorded.

### Trash artwork and raster assets

Approved SVG artwork remains the canonical editable source. The compositor does not contain a runtime SVG renderer, and librsvg is not approved as a core runtime dependency because of its wider Cairo/GLib/Pango dependency footprint.

The generated icon directory currently contains its README but no generated raster icon set. A deterministic SVG-to-RGBA8 host pipeline is now committed: existing rsvg-convert produces temporary PNGs and a bounded Python standard-library converter validates and strips them to tightly packed raw RGBA8. It creates 64, 96, 128 and 192 pixel variants for a 64-logical-pixel icon. The pipeline has not yet been executed or visually validated.

## Linux and Debian integration

### JPEG codec

The TurboJPEG ABI probe has exercised:
- Debian 13 Trixie: `libturbojpeg0-dev` and `libturbojpeg0`, version `1:2.1.5-4`.
- Fedora: `libjpeg-turbo-devel` and `turbojpeg-devel`, version `3.1.4.1-2.fc45`.

Both container jobs passed the probe's compile, SONAME and real-wallpaper decode checks. The runtime library is distinct from the development headers. Keep the native adapter isolated; do not add a general-purpose Rust image stack for the initial JPEG-only requirement.

### Shader compiler

`glslang-tools` is available in Debian 13 Trixie and is build-time only. Fedora's package is `glslang`. Do not include shader compiler tools in the installed runtime image without a separate reason.

### Linux desktop interoperability

Use established FreeDesktop/XDG conventions where appropriate:
- Desktop Entry for application metadata and launch.
- Icon Theme conventions for icon lookup and fallback.
- Shared MIME-info for file type identification.
- Thumbnail Managing Standard and registered thumbnailers.
- Trash specification and XDG Base Directory locations.
- Wayland protocols and Smithay/libinput for surfaces, input and outputs.

These specifications do not supply a complete desktop shell or native desktop-icon renderer; AureaOS owns those product-level components.

## Ownership boundaries

- **Aurea Background:** wallpaper selection, layout and policy. Decode delegated to Aurea Media; render consumes completed texture state.
- **Aurea Media:** bounded in-process decode/asset worker. No compositor, Smithay or Vulkan dependency.
- **Aurea Desktop Items:** item model, placement, hit testing and interaction state. Not a file manager or Dock.
- **Aurea Files:** filesystem operations, MIME/open-with, thumbnails and Trash service.
- **Aurea Shell:** Dock, launcher, panels, notifications and user-facing controls.
- **Aurea Compositor:** Wayland surfaces, outputs, pointer routing, hit-test handoff, Vulkan composition and presentation.

## Rendering order

1. Clear output to the selected theme fallback color.
2. Draw the selected wallpaper texture using per-output fit geometry and clipping.
3. Draw native desktop items once their raster assets and geometry are ready.
4. Draw mapped Wayland surfaces in stacking order.
5. Draw the native pointer/cursor above the scene.
6. Apply the existing output color pass and present through DRM/KMS.

Do not regress Night Light, output transforms, fractional scaling, scanout synchronization or GPU resource lifetime while extending the pipeline.

## Next implementation sequence

1. Execute and visually inspect the deterministic SVG-to-RGBA8 pipeline on Fedora; verify all 24 outputs, alpha edges, theme contrast and optical alignment.
2. Add a small compositor-owned desktop-asset description/cache with bounded raw RGBA dimensions and asynchronous Vulkan upload. Do not overload the Wayland SHM texture cache.
3. Connect the per-output icon geometry module to the desktop texture cache and draw items between wallpaper and Wayland surfaces.
4. Connect the existing `DesktopHitTest` logical geometry to pointer input only after the icons are visible. Preserve Wayland surface focus/raise precedence and forward events normally.
5. Add selection, single/double-click and context-menu semantics through explicit desktop interaction contracts; keep file opening and application launch in Aurea Files/Shell, not in the compositor.
6. Review active-grab completion when an output disappears, then validate hotplug/session pause/resume on hardware.
7. Run the agreed grouped Cargo formatting/check/test and visual validation on Fedora. Do not run builds/tests for every small source correction.

## Explicit non-goals

- No ISO generation during desktop development.
- No X11, XWayland or OpenGL.
- No GNOME/KDE desktop dependencies.
- No runtime SVG engine added solely for static icons.
- No permanent background daemon without measured need.
- No copying COSMIC implementation code or assets.
- No unrelated compositor or Vulkan restructuring.

## Verification limits

Repository inspection and the recorded container ABI probe do not prove that the desktop boots, that the wallpaper is visible on the user's hardware, that all Vulkan paths work on the AMD Stoney GPU, or that hotplug/session restoration is correct in a live session. Those remain explicit validation tasks.
