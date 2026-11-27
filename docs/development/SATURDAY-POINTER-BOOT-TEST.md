# Saturday Pointer-on-Screen Acceptance Test

Target: Saturday 28 November 2026  
Scope: boot the Live USB to one graphical output showing only the default wallpaper and a visible, moving physical mouse pointer. No Welcome window, desktop icons, dock, launcher, menus, settings or additional UI are part of this milestone.

## What the source currently implements

- The Wayland seat pointer is initialized during server construction.
- The compositor starts with Smithay's default named cursor status.
- The built-in 32×32 high-contrast arrow is rasterized as RGBA and uploaded once to the Vulkan texture cache.
- Every render scene combines the current seat pointer location with the cursor image status.
- The Vulkan surface compositor computes output-relative cursor geometry and draws the native arrow after desktop surfaces and the desktop context menu.
- Relative libinput motion from registered mouse/touchpad devices updates the Wayland seat pointer, constrains it to enabled output geometry, and requests output frames.
- Startup centers the pointer on the first enabled output and requests a frame.
- The startup path now fails clearly if active outputs exist but the pointer cannot be placed on the primary output.

These are source-level findings. They do not prove the native Fedora hardware path has succeeded; only a boot and physical mouse test can establish that.

## First-frame wallpaper acceptance

The default `aureaos-blue-flow.jpg` is embedded in the compositor binary. The
first output frame starts bounded worker decoding and a fence-backed Vulkan
texture upload. The event loop must continue requesting frames while either
operation is pending; otherwise an idle desktop can remain on the clear-color
background forever after its first frame.

The readiness path now includes pending wallpaper decode and upload work in the
same frame-polling condition used by native cursor assets. The Live systemd unit
sets `AUREAOS_LIVE_MINIMAL=1`; the desktop factory uses this flag to skip the
Welcome Wayland client, leaving the compositor wallpaper and native pointer as
the only intended visible content. Normal development sessions retain Welcome.

On the physical boot:

1. Confirm the default wallpaper appears without moving the mouse or opening a window.
2. Confirm it fills the active output without stretching or leaving unintended margins.
3. Confirm the pointer is visible over the wallpaper on the first desktop frame.

If the wallpaper does not appear, inspect the decode worker, upload-fence state,
and the `Wallpaper texture cache` diagnostics before changing image assets.

## Before the native run

On Fedora, from the repository root:

```bash
git switch main
git pull --ff-only origin main
cargo fmt --all -- --check
cargo check -p aurea-compositor --features drm
cargo test -p aurea-compositor --features drm
```

Fix all reported compile, formatting and test failures before proceeding. Do not generate an ISO until the native run is successful.

## Native boot and physical test

Run the existing diagnostic launcher from a normal local user session:

```bash
bash tools/run-aurea-native-test.sh
```

It builds the desktop executable, captures PCI/DRM/input/libinput/Vulkan inventory, launches the native DRM/Vulkan desktop, and collects the run diagnostics. It expects a valid `XDG_RUNTIME_DIR` and an interactive local session.

Once the graphical output is visible:

1. Confirm no Welcome window or other desktop UI is covering the wallpaper.
2. Confirm the wallpaper fills the active monitor without stretching or unintended margins.
3. Confirm the white arrow with dark outline is visible without moving the mouse.
4. Move the physical mouse slowly in all four directions. The arrow must follow continuously.
5. Move to each edge and corner. It must remain inside the display.
6. Exit with Ctrl+C and allow the diagnostic script to collect the report.

## Acceptance criteria

The Saturday minimum is met only when all are true:

- Native desktop reaches a visible frame without a compositor startup/render error.
- The default pointer arrow is visible on that frame before any mouse movement.
- Physical relative mouse motion moves the on-screen arrow.
- Pointer motion remains within enabled output bounds.
- The Live session contains no Welcome window or unrelated UI.
- The run exits cleanly or the reason for failure is captured in the diagnostic report.

If the cursor is absent, do not change themes or cursor assets first. Inspect in this order:

1. `AureaOS pointer: startup capability ready` — seat capability and initial placement.
2. `AureaOS input: device added` / `Pointing device connected` — libinput discovery and classification.
3. `AureaOS render: pending output` and `render_output completed` — frame scheduling and presentation.
4. Vulkan errors involving native cursor texture upload, descriptor binding, draw submission or KMS presentation.
5. The current cursor status: `Hidden`, `Named` or client-provided `Surface`.

If the arrow is visible but does not move, inspect libinput device discovery, the `PointerMotion` event branch, registration state, output bounds and session-active state.

## Changes in this readiness pass

- Added tests that the default named cursor becomes a native cursor in a render scene at the seat pointer location; missing pointer locations and explicitly hidden cursor states do not create a native cursor.
- Added Vulkan cursor-geometry tests proving the default arrow intersects the active output and is clipped out on a different output.
- Made startup placement failure explicit when active outputs exist, rather than silently leaving the cursor at a potentially invalid default coordinate.
- No build, formatter, test, Vulkan run or physical-device validation has been performed in this source-editing pass.
