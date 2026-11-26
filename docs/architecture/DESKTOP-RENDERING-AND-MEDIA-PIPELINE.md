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

**Validation boundary:** the repository contains a native DRM test from 2026-11-21 (build revision `63196256`) that recorded an active libseat session, one discovered DRM device/output, two completed output-render calls on the `cpu-compatibility` path, and a Welcome Wayland client that received XDG configure and submitted its initial SHM frame. This is evidence that that earlier source revision reached the frame-submission path; it is not a screenshot or independent confirmation that the image was visibly presented on the physical monitor, and it does not validate the current source after the subsequent pointer/session changes. A separate 2026-11-20 run failed all negotiated Vulkan/KMS linear scanout allocations with `ERROR_FORMAT_NOT_SUPPORTED`. The TurboJPEG ABI probe passed in Debian 13 and Fedora GitHub Actions containers; that validates the exercised native ABI path, not the complete Rust production integration. The current Rust integration and Vulkan/KMS presentation still require the agreed grouped verification on Fedora.

### Vulkan composition and failure handling

The existing renderer retains separate ownership for Wayland SHM textures, wallpaper textures and the native cursor. Uploads are fence-backed and asynchronous where implemented. Descriptor resources remain frame-scoped until GPU completion is established.

Vulkan `ERROR_DEVICE_LOST` is propagated as a fatal typed condition through the reviewed wallpaper, SHM, cursor-upload and scanout paths. The current policy is fail-stop: do not attempt a CPU fallback that still depends on the lost logical device, and do not retire GPU/KMS resources while completion is uncertain. Full device recreation is not implemented.

### Native desktop items

The technology-independent desktop API already defines:

- Stable `DesktopItemId`, item kinds (File, Folder, Application, Trash) and logical positions.
- Desktop preferences and visual grouping.
- `DesktopRect` and deterministic `DesktopHitTest`, including overlap ordering and logical-coordinate semantics.
- A compositor-owned `DesktopItemsSnapshot` captured into `RenderScene`.

The native Vulkan composition path does **not** yet draw desktop item icons. A first per-output geometry module converts each item's global logical position into physical output geometry, applying integer/fractional scale, output transform and clipping. It uses a 64-logical-pixel icon destination and skips invalid/off-output entries independently. Desktop hit testing is connected to primary pointer presses on the empty desktop: mapped Wayland surfaces retain focus/raise precedence, and a click outside a window selects or clears the item in Shell state. The compositor-to-shell channel also publishes a shared selection projection; the shell mirrors it into `DesktopState`, and each `DesktopItemsSnapshot` copies the selected identifier for the frame. Per-output `DesktopItemDrawGeometry` carries an `is_selected` flag. This is data plumbing only: no Vulkan draw pass consumes that flag yet, so no visible highlight or native icon rendering is claimed. A separate monotonic click tracker recognizes a same-item double-click and emits a semantic activation intent. Actual file/application launching is not implemented. Do not treat source tests as proof of end-to-end interaction on hardware.

### Pointer, output hotplug and session lifecycle

- Relative motion, buttons, axes/scroll and touchpad gestures arrive through libinput/Smithay.
- Pointer coordinates are global logical coordinates and are constrained to the union of enabled output rectangles. Gaps resolve to the nearest output edge.
- Primary-button presses focus/raise a Wayland toplevel when a surface is hit; otherwise they run desktop item hit testing. The event is still forwarded through the Wayland pointer. Double-click activation is emitted as a semantic intent only; the Files/Shell launch service is not connected yet.
- Native cursor geometry is resolved per output using logical position, cursor scale, target scale/fractional scale and output transform.
- Output reconciliation refreshes pointer focus and requests frames. DRM-device removal unregisters its outputs and attempts to place the pointer on the primary remaining output.
- On libseat pause, input events are ignored; KMS surfaces are cleared before graphics resources are released. On activation, GBM/Vulkan/KMS and deferred outputs are restored before the pointer is centered on an enabled output.

**Known limitation:** centering is deliberately skipped while a client owns an active pointer grab. The pointer is constrained on subsequent physical motion, but the end-of-grab transition after removal of the output under the pointer still deserves a focused lifecycle review. No hardware hotplug/session-switch validation is recorded.

### Trash artwork and raster assets

Approved SVG artwork remains the canonical editable source. The compositor does not contain a runtime SVG renderer, and librsvg is not approved as a core runtime dependency because of its wider Cairo/GLib/Pango dependency footprint.

The generated icon directory currently contains its README but no generated raster icon set. A deterministic SVG-to-RGBA8 host pipeline is now committed: existing rsvg-convert produces temporary PNGs and a bounded Python standard-library converter validates and strips them to tightly packed raw RGBA8. It creates 64, 96, 128 and 192 pixel variants for a 64-logical-pixel icon. The pipeline has not yet been executed or visually validated.

A dedicated compositor-side Vulkan texture cache has now been added for native desktop artwork. Its identity is independent from Wayland `ObjectId` values and includes an approved asset identifier, revision and raster pixel size. It enforces a 32-texture / 16 MiB resident budget and a two-batch / 8 MiB in-flight upload budget, validates square dimensions up to 256 pixels, and retains pending resources until upload fences complete. This is cache infrastructure only: no generated icon bytes are available yet, and the cache is not connected to the frame renderer or desktop item geometry.

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


### Native cursor texture ownership

The compositor-owned arrow texture and its asynchronous upload fence are isolated in `vulkan/native_cursor_texture.rs`. The resource owns only its texture and pending upload lifecycle; `surface_pipeline_cache.rs` delegates cursor preparation and exposes the ready texture. This keeps cursor upload/error handling out of the general surface-pipeline coordinator.

### Trash service module boundaries

The home Trash implementation keeps filesystem operations in `trash/service.rs`. Calendar conversion for the freedesktop `.trashinfo` deletion date is isolated in `trash/timestamp.rs`; metadata parsing and serialization remain in `trash/metadata.rs`. Public service behavior is covered from `components/aurea-desktop/tests/trash_service.rs`, rather than embedding a large test module inside the service implementation. This keeps the service focused on directory operations, entry lifecycle and state snapshots.

### Desktop icon asset resolver

A compositor-owned descriptor module now maps the existing desktop model's Trash item to the six approved SVG-derived variants:

- Light/Dark appearance × Empty/PartiallyFull/Full contents.
- Raster variants: 64, 96, 128 and 192 pixels for the 64-logical-pixel destination.
- The resolver chooses the smallest available raster that covers the effective output scale: 1x up to 1.0, 1.5x up to 1.5, 2x up to 2.0, then 3x. Non-finite scale values fall back conservatively to 1x.
- Stable cache identity: asset ID, content revision and raster pixel size.
- Repository-relative generated asset path matching the host export script.
- RGBA8 bytes are passed through the existing checked `ShmBufferSnapshot::from_rgba8` constructor before they can become a Vulkan cache request.

The visual state is now a technology-independent `DesktopTrashState` in `aurea-desktop-api`. The desktop model owns it and `DesktopItemsSnapshot` captures it for the render frame. Aurea Shell reuses the same enum, and the Trash service exposes an explicit conversion from its `TrashVisualState`. The compositor still does not query the filesystem: the application composition root must pass the service snapshot into the shared desktop model. No watcher or automatic refresh loop is claimed.

File, Folder and Application entries deliberately return no icon descriptor because their canonical AureaOS artwork and MIME/application lookup are not yet approved or integrated.

This is metadata and validation infrastructure only. The 24 raw RGBA8 files have not been generated, loaded or visually inspected. No icon is drawn yet. The shared texture key lives in the desktop module so the model/asset contract remains available when the optional Vulkan feature is disabled.

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
2. Generate and visually inspect the static RGBA8 assets on Fedora; do not commit unverified binary output.
3. Add a small compositor-owned asset descriptor that maps desktop item kind/theme/state to approved raw RGBA bytes and creates validated snapshots.
4. Connect the per-output icon geometry module to the dedicated texture cache and draw items between wallpaper and Wayland surfaces.
5. Connect the existing `DesktopHitTest` logical geometry to pointer input only after the icons are visible. Preserve Wayland surface focus/raise precedence and forward events normally.
6. Add selection, single/double-click and context-menu semantics through explicit desktop interaction contracts; keep file opening and application launch in Aurea Files/Shell, not in the compositor.
7. Review active-grab completion when an output disappears, then validate hotplug/session pause/resume on hardware.
8. Run the agreed grouped Cargo formatting/check/test and visual validation on Fedora. Do not run builds/tests for every small source correction.

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


### Native cursor error contract

The native cursor upload lifecycle reports an explicit empty-result error rather than synthesizing a Vulkan driver error. The surface composition layer consumes the extracted `NativeCursor` cache error and retains the underlying SHM texture error classification, including device loss. This keeps upload-contract failures distinct from Vulkan command failures.


### Initial Trash state flow

At session startup, the Aurea Desktop composition root reads one snapshot from the home Trash service and passes only the resulting `DesktopTrashState` through `WaylandCompositorFactory` to the compositor’s shared desktop item model. The compositor never scans Trash directories or owns filesystem policy. If the initial service read fails, startup continues and records a warning.

This is an initial snapshot only. Live refresh after file deletion/restoration still requires a session-owned watcher or polling service; the current implementation does not claim automatic updates.


### Desktop item selection input flow

The primary pointer-button path gives mapped Wayland surfaces precedence. When no client surface is hit, the compositor converts the global pointer location to logical integer coordinates and uses the shared `DesktopHitTest` contract with the desktop icon logical dimensions. It emits either `DesktopItemSelected { id }` or `DesktopSelectionCleared` through the existing compositor-to-shell interaction channel. The channel carries a small `DesktopSelectionController` backed by `Arc<RwLock<Option<String>>>`: successful selection events publish the frame projection immediately, avoiding a race with the click-triggered redraw, while `ShellRuntime` mirrors the same value into `DesktopState`. The compositor only reads an owned string snapshot; it never holds the lock during rendering.

`DesktopItemsSnapshot` copies the selected identifier once per frame, and per-output geometry marks the matching item with `is_selected`. This establishes the path from Shell state to Vulkan frame preparation, but the Vulkan draw pass does not consume the flag yet. The visual selection treatment remains pending. A compositor-owned `DesktopClickTracker` recognizes a second click on the same item within 400 ms and within 4 logical pixels. It uses monotonic `Instant` timestamps and has no timer or polling loop; clicking a window or empty desktop cancels the pending first click. A recognized double-click emits `DesktopItemActivated { id }` after the selection event.

The shell retains activation as a semantic pending action, but does not launch files or applications. Resolving desktop entries, permissions and launch policy belongs to the Files/Shell service layer and is not implemented by this tracker. No icon drawing or visible selection highlight is claimed yet.

## Desktop item presentation ownership contract

The desktop presents every supported item category through one native item pipeline. Trash is one item kind, not a special renderer or the definition of the desktop architecture.

- **Aurea Shell / Desktop experience:** owns user-facing presentation policy: exposed entries, arrangement preferences, labels, selection UX, drag/drop policy, context-menu intent and activation intent. It does not issue Vulkan commands or own GPU resources.
- **aurea-desktop-api:** owns technology-independent contracts: stable IDs, item kinds, resource references, logical positions, preferences, selection and activation events. It must not depend on Smithay, Vulkan, filesystem implementations or Fides internals.
- **Aurea Files / filesystem services:** discover entries and provide authoritative metadata, MIME type, thumbnails and file operations. Fides is the user-facing file manager, not the desktop icon renderer.
- **Trash service:** owns Trash contents, state calculation, delete/restore/empty operations and refresh notifications. It exports semantic state/data; it does not draw the Trash icon.
- **Icon and theme asset service:** resolves approved artwork for item kind, MIME/application identity and theme, and supplies validated raster data. It does not decide placement or perform file operations.
- **Aurea Compositor:** owns output-specific physical geometry, clipping, GPU texture lifetime, Vulkan draw commands, frame ordering and presentation. It consumes immutable item snapshots and prepared visual assets. It must not scan directories, infer MIME types, count Trash entries, launch applications or implement file-management policy.

The frame data flow is one-way: Shell and services publish semantic desktop state through aurea-desktop-api; the compositor captures an immutable snapshot, resolves output geometry and draws prepared assets. Pointer hit testing may identify an item and return a semantic selection/activation event, but opening a file or launching an application remains a Shell/Files responsibility.

### Common item rendering contract

The native item renderer must support File, Folder, Application and Trash through one common visual-entry pipeline:
- Trash artwork depends on Empty/PartiallyFull/Full semantic state and theme.
- Folder artwork uses a canonical folder asset.
- File artwork uses authoritative MIME metadata, not filename extension alone; a generic-file fallback is required.
- Application artwork is resolved from desktop-entry/application identity, with a generic application fallback.

The renderer receives a resolved asset key and immutable pixel snapshot. It does not inspect URI strings to guess file types. Asset I/O, decoding and raster preparation stay outside the frame-critical path; Vulkan uploads remain bounded, asynchronous and fence-owned. A missing optional icon must not prevent other items, windows or the pointer from rendering.

### Current implementation boundary and next sequence

The shared API defines all four item kinds, but the compositor snapshot and geometry are preparatory. The icon resolver currently resolves Trash only; File, Folder and Application have no approved asset mapping. The dedicated Vulkan icon cache is not fed by generated assets and is not connected to the draw pass. Selection and double-click intent reach Shell, but there is no visible icon or selection highlight yet, and no file/application launch implementation.

Repository-relative Trash asset paths are suitable for source export, not a finalized installed runtime lookup contract. Before frame integration, define one explicit asset-root/packaging contract and pass it into an asset loader. Do not rely on process working directory or perform filesystem reads inside render_frame.

1. Finalize the installed asset-root contract and generate/visually inspect the 24 Trash RGBA8 variants on Fedora; do not add unverified binaries.
2. Add a small asset loader that reads validated raw RGBA8 data outside the render loop and reports missing/corrupt assets without aborting the desktop session.
3. Complete item-to-asset mapping for Trash, Folder, File and Application using MIME/application metadata supplied by services.
4. Prepare per-output geometry and texture requests from the immutable snapshot, then connect the dedicated bounded Vulkan icon cache.
5. Extract icon drawing and selection treatment into focused modules; keep surface_compositor.rs an orchestrator. Draw wallpaper, desktop items/selection, Wayland surfaces, pointer, then display color.
6. Add live item discovery and Trash-state updates through service/API notifications. Keep filesystem watchers and policy outside the compositor.
7. Connect activation to Shell/Files operations. The compositor emits semantic intents only.
8. Run one grouped Fedora verification/visual session after this implementation block; do not build, test or run rustfmt after every small edit.


## First usable desktop milestone — 24 November 2026

The immediate implementation target is intentionally narrower than the long-term item contract above. The first usable desktop must contain only:

- the desktop background (solid color or the currently selected wallpaper);
- the native pointer, with movement, primary/secondary buttons, double-click recognition and wheel input;
- one Trash icon at the upper-left desktop position, already defined by the desktop model as global logical coordinates (32, 32);
- visible selection feedback when the Trash item is clicked.

Files, folders, application icons, context menus, drag-and-drop and opening/launching items are explicitly deferred. The shared item model may retain its future-facing variants, but this milestone must not implement speculative rendering or filesystem behavior for them.

### Implementation gate

The Trash resolver and bounded Vulkan cache exist, but the 24 generated RGBA8 variants are not yet present/verified. The export script must be run on Fedora and the outputs inspected before the icon is integrated. Runtime loading must use an explicit installed asset root and happen outside the frame-critical path; repository-relative paths and current-working-directory assumptions are not acceptable.

The renderer integration must preserve the common frame order: background/wallpaper → Trash artwork and selection treatment → Wayland surfaces → pointer → display-color transform → presentation/readback. Both direct Vulkan and CPU-upload compatibility paths already share the composition path, so the icon pass must be inserted there once, not duplicated per backend.

No claim is made that the Trash is visible or that the desktop is hardware-validated until the asset pipeline and draw pass are connected and the grouped Fedora visual test succeeds. Build, tests and rustfmt remain grouped into the agreed verification session rather than being run after each edit.


### Desktop asset loader boundary

Added a small loader that accepts an explicit asset root, rejects absolute/parent-traversal paths and validates loaded bytes through the existing RGBA8 snapshot constructor. The application composition root has not selected the installed runtime root and the renderer does not call this loader yet. It is staged groundwork only; no frame-loop filesystem access or new dependency was introduced. No build, tests, rustfmt or CI were run.

### Pointer selection and context-menu status — 25 November 2026

The native pointer-button handler now routes clicks on the empty desktop through `select_desktop_item_at`. Primary click selects/clears and retains the existing same-item double-click activation intent. Secondary click selects/clears first, then sends `ShowContextMenu` with global logical pointer coordinates. Mapped Wayland windows keep precedence and continue to own their context menus.

The Shell model records `DesktopPanel::ContextMenu` and its requested position, but there is not yet a rendered menu surface or actionable menu entries. Do not claim the context menu is visible. The selection visual treatment also remains pending in the Vulkan desktop-item draw pass. Commit: `80483c23`.


### Desktop context menu placement and presentation boundary

The compositor-to-shell interaction now carries the pointer anchor in global logical coordinates, and the shell retains it while the context-menu panel is active. The shared desktop API also provides output-constrained placement: the presentation layer supplies the measured menu size and the logical bounds of the output under the pointer; placement keeps the complete menu visible by shifting it left/up at right/bottom edges. It rejects zero-sized or oversized layouts rather than returning clipped or overflowing geometry.

This paragraph records the earlier presentation proposal and is superseded by the compositor-owned Vulkan overlay decision made on 25 November 2026. Shell remains the semantic owner of the panel state and commands; it does not need a separate Wayland client for this first native context menu. The compositor receives only an immutable layout/theme snapshot and draws approved raster artwork through its existing Vulkan texture pipeline. The four menu commands remain semantic intents: their real service dispatch is still pending, so visual presentation does not mean the actions are complete.


### Desktop context-menu Vulkan overlay — 25 November 2026

The desktop context menu now follows the compositor-owned native artwork path rather than creating a client Wayland surface:

- `RenderScene` carries the output-constrained logical menu layout and the current desktop theme.
- `VulkanSurfacePipelineCache` loads the selected RGBA8 asset lazily on first use from `AUREA_HUMAN_INTERFACES_ROOT` (default `/usr/share/aureaos/human-interfaces`). The file is validated by the existing explicit-root asset loader, retained as a CPU snapshot, and uploaded through the bounded desktop-artwork Vulkan cache. Frame rendering does not read the filesystem.
- The existing premultiplied-alpha textured pipeline is reused. The overlay is drawn after mapped Wayland surfaces and before the cursor, with output-local physical coordinates derived from each target's logical origin and effective scale. The scissor clips the 256px artwork canvas to the current output.
- Development checkouts must set `AUREA_HUMAN_INTERFACES_ROOT` to the repository's `aurea-human-interfaces` directory. Installed packages use `/usr/share/aureaos/human-interfaces`.
- The menu is still conditional on the generated light/dark RGBA files. Run `tools/assets/render-context-menu.sh` before launching the compositor. Missing artwork is reported once and leaves the menu artwork undisplayed; it does not abort compositor startup.
- Source-level geometry tests cover 1x, 2x and unrelated-monitor clipping. No Cargo build, rustfmt, check, tests or visual validation have been run. The next grouped Fedora session must generate the RGBA assets, run the agreed verification suite, and inspect both themes at mixed output scales.
- The shared PNG exporter converts PNG's straight alpha to premultiplied RGBA8 to match the existing Vulkan blend state (`ONE`, `ONE_MINUS_SRC_ALPHA`). This applies consistently to compositor-owned icon artwork, not only the context menu.


## Native pointer rendering

The compositor owns a small high-contrast 32×32 RGBA arrow for the default named cursor. RenderScene derives its top-left origin from the seat pointer location and the arrow hotspot; output geometry applies each monitor's logical origin, integer or fractional scale, transform and clipping. A client-provided Wayland cursor surface instead follows its committed hotspot and is composited above desktop overlays; the compositor-owned arrow is drawn last when the cursor status is a named cursor.

The native arrow texture is initialized once and cached for the Vulkan device lifetime. Its first upload is completed before the first frame that requires it, ensuring the startup pointer-centering frame cannot omit the arrow while waiting for another input event. The cursor is positioned by the same seat pointer location updated by libinput motion, and each accepted relative motion requests output frames. These are source-level guarantees; physical movement, click, wheel, multi-monitor and hotplug behavior still require the grouped Fedora validation.


### Native Xcursor rendering — 26 November 2026

- `RenderScene` carries the named Smithay cursor role, global pointer anchor and desktop Light/Dark theme.
- `VulkanNativeCursorTexture` resolves Xcursor files from `AUREA_CURSOR_THEME_ROOT` (default `/usr/share/icons`), decodes with the bounded internal parser, selects the closest raster size and retains the uploaded frame textures in the per-device cache.
- The native cursor draw uses the decoded image dimensions and per-asset hotspot. Missing files fall back to a raster-size-adjusted built-in arrow.
- Debian Live packaging builds both SVG themes with `rsvg-convert` and `xcursorgen`, installs them in `/usr/share/icons`, and validates the canonical cursor payload.
- Current limitation: first-use file read/decode/upload still occurs synchronously on the render path; all selected frames are uploaded but only frame zero is drawn. Animation scheduling, asynchronous preparation, cache-budget review and physical fractional/high-DPI validation remain open. No build or test was run for this source change.

### Asynchronous native cursor loading and frame animation — 26 November 2026

- A bounded `aurea-xcursor-decode` worker performs file I/O, parser work and fallback raster generation away from the render callback.
- The compositor polls worker results and fence-backed Vulkan uploads. Cursor textures are exposed only after upload completion.
- Each Xcursor frame retains independent size, hotspot and delay metadata. Frame selection uses monotonic elapsed time.
- The event loop requests 16 ms output frames while the animated `busy` cursor is selected and during a three-second first-use warm-up period. The warm-up is explicitly restarted after DRM/GPU discovery, output publication and successful startup pointer placement; this prevents slow device initialization from consuming the window before the first frame. The compositor requests output frames immediately so asynchronous decoding/upload can complete while the pointer is idle.
- Cache byte limits/LRU, reduced-motion handling, `left_ptr_watch` animation and physical validation remain open. No Cargo build or test has been run for this change.

- Follow-up: native cursor cache residency is capped at 32 assets / 16 MiB with LRU eviction. A single animation is capped at 4 MiB; oversized sequences keep their first frame.
- Added GPU-independent tests for per-frame delay selection, frame boundaries, loop wraparound and zero-delay normalization.
- The current manifest defines `busy` as 12 frames / 1100 ms; `left_ptr_watch` is a static image, so it does not need a frame scheduler.

### Native cursor locate trail — 26 November 2026

- A bounded three-position history is captured only while the compositor's shake-to-locate animation is active. Samples are throttled to 24 ms and fade over 220 ms.
- Render scenes carry immutable, newest-first snapshots. Output-local geometry reuses the native cursor's hotspot, effective output scale, transform and clipping rules.
- A dedicated fragment shader and cached Vulkan pipeline apply per-impression opacity through fragment push constants. Impressions are submitted oldest-first and the live native cursor is drawn last.
- The effect currently applies to the compositor-owned named cursor. Client-provided Wayland cursor surfaces are not converted into trail textures. Runtime visual validation and Fedora build/test verification remain pending.
