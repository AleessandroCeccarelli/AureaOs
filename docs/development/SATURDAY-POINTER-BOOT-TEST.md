# Saturday Pointer-on-Screen Acceptance Test

Target: Saturday 28 November 2026  
Scope: the first visible and moving AureaOS pointer on the native desktop. Do not expand this test into unrelated desktop features.

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

Once the desktop is visible:

1. Confirm the white arrow with dark outline is visible without moving the mouse.
2. Move the mouse slowly in all four directions. The arrow must follow continuously.
3. Move to each edge and corner. It must remain inside the display.
4. Test left click, right click and double click.
5. Scroll vertically and horizontally if the device supports both.
6. Disconnect and reconnect the mouse; confirm motion resumes without restarting AureaOS.
7. If a touchpad is available, confirm its motion does not disable or strand the pointer.
8. Exit with Ctrl+C and allow the diagnostic script to collect the report.

## Acceptance criteria

The Saturday minimum is met only when all are true:

- Native desktop reaches a visible frame without a compositor startup/render error.
- The default pointer arrow is visible on that frame before any mouse movement.
- Physical relative mouse motion moves the on-screen arrow.
- Pointer motion remains within enabled output bounds.
- Left/right button events and wheel events reach the desktop/Wayland seat.
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
