# SPDX-License-Identifier: GPL-3.0-or-later

# AureaOS — Input Event Coverage Audit

**Status:** source/API audit, not runtime validation  
**Backend baseline:** Smithay 0.7 / libinput  
**Last reviewed:** 24 November 2026

## Purpose

Record which input event families are represented by Smithay 0.7 and how AureaOS currently routes them. Device capability detection must not be confused with event delivery or feature support.

## Smithay 0.7 event surface

Smithay 0.7 `backend::input::InputEvent` includes:

- Device lifecycle: `DeviceAdded`, `DeviceRemoved`
- Keyboard: `Keyboard`
- Relative and absolute pointer motion: `PointerMotion`, `PointerMotionAbsolute`
- Pointer buttons and axes: `PointerButton`, `PointerAxis`
- Swipe, pinch and hold gesture begin/update/end events
- Touch down, motion, up, cancel and frame events
- Tablet tool axis, proximity, tip and button events
- Switch changes: `SwitchToggle`
- Backend-specific `Special`

**There is no TabletPad event variant or TabletPad event trait in this Smithay 0.7 input event enum.** The device capability `DeviceCapability::TabletPad` exists, but that capability alone does not provide pad button/ring/strip event delivery through this enum. Do not add guessed variants or claim pad control support without a different verified API/backend path.

## AureaOS routing status

| Event family | Current handling | Status |
|---|---|---|
| Device add/remove | Device registry, touchpad setup, tablet-tool cleanup | Routed |
| Keyboard | Compositor keyboard handler | Routed |
| Relative pointer motion | Acceleration/settings and compositor pointer handler | Routed |
| Pointer buttons | Compositor button handler | Routed |
| Pointer axes | Compositor scroll handler | Routed |
| Swipe/pinch/hold | Gesture handlers | Routed |
| Touchscreen | Calibrated output extent and touch handlers | Routed |
| Tablet tool | Proximity, axes, tip and tool buttons | Routed |
| Absolute pointer motion | Output-local mapping on exactly one enabled output; multi-output layouts are safely ignored pending explicit assignment | Partially routed |
| Switch toggle | State-change diagnostic only; no lid/tablet-mode policy | Not implemented |
| Tablet pad controls | No corresponding Smithay 0.7 InputEvent path | Not available through this event API |
| Special backend events | No AureaOS policy | Ignored |

## Source changes in this audit

- Connected `PointerMotionAbsolute` to the compositor. A single enabled output provides an unambiguous target and allows output-local coordinates to be mapped into global logical coordinates. Multi-output layouts remain blocked until per-device output assignment is available.
- Added explicit `SwitchToggle` handling. It records device, switch and state. Power/session policy for lid close and tablet mode remains outside the compositor input router.
- Kept the final wildcard for future/non-actionable backend variants; the known absolute and switch cases are no longer silently swallowed.

## Follow-up implementation sequence

1. Add an explicit per-device output assignment contract for absolute pointers so multi-monitor routing can be enabled without guessing.
2. Define ownership and policy for lid-close and tablet-mode switches in the session/power-management layer; the compositor should consume resulting state, not own power policy.
3. Revisit tablet-pad support only if Smithay adds a supported event path or AureaOS adopts a separately verified libinput integration that does not fork/patch blindly.
4. Keep tool buttons distinct from tablet-pad buttons: current tablet-tool button routing does not imply pad support.

## Verification

Static source/API review only. No build, Cargo check, rustfmt, tests or hardware validation were run. Those remain grouped for the agreed Friday Fedora session, with Debian 13 compatibility as the dependency baseline.


## Device inventory distinction — 2026-11-24

AureaOS now distinguishes `InputDeviceKind::Tablet` (a pen/tablet-tool device with routed tool events) from `InputDeviceKind::TabletPad` (a pad-only device). Pad-only devices remain visible in the pointing-device inventory and retain their discovered pad capability counts, but they are not accepted by pen calibration or tablet-to-output assignment APIs. Devices exposing both `TabletTool` and `TabletPad` are classified as `Tablet`, with pad capabilities retained in the descriptor. This is inventory accuracy only; Smithay 0.7 still provides no pad-control event route.

## Tablet output disconnect policy — 2026-11-24

Pen-tablet output mappings now follow the same no-surprise rule as touchscreen mappings. When an explicitly assigned output disconnects, the assignment is retained as unavailable and the tablet resolves to no target; it is not silently changed to the full desktop or another remaining output. An explicit reassignment restores routing, while an explicit unassignment restores the default full-desktop mode. Device removal clears both the assignment and unavailable state. This is a source-level policy change; runtime behavior remains pending grouped Fedora validation.

## Pen-tablet motion and monitor gaps — 2026-11-24

When a tablet is mapped to the full desktop, the rectangular calibration extent can span empty gaps between non-adjacent or offset monitors. Tablet coordinates are now constrained against the actual enabled output rectangles after calibration. A point falling in a gap is moved to the nearest valid output edge, following the same geometry policy already used by relative pointer motion. For a tablet explicitly mapped to one output, the candidate geometry is restricted to that output, so the pen cannot escape its assigned display. This does not change touchscreen transforms. Generic absolute-pointer routing is now available only on single-output desktops; multi-output routing still awaits explicit assignment.


## Touch cancellation semantics — 2026-11-25

- Smithay 0.7 provides `TouchHandle::cancel` as a seat-wide operation; it does not expose per-device cancellation.
- AureaOS now applies an explicit **seat-wide cancellation policy**: a `TouchCancel` is acted on only when the originating device owns at least one active slot. The compositor then clears every active physical-to-seat slot mapping and emits exactly one `TouchHandle::cancel` for the seat.
- This intentionally cancels concurrent contacts from other touchscreens too. That is required to keep the server's aggregated Wayland seat state consistent with the protocol event. Their physical contacts may still be down, but no stale slot remains; subsequent motion/up events are ignored until a fresh down allocates a new slot.
- A cancel from a device with no active slot is ignored, so a spurious/late event cannot cancel another device's live sequence.
- Device removal reuses the same cancellation path before clearing the remaining per-device settings and mappings.
- The slot registry exposes `clear_all` to make the seat-wide reset explicit and has a unit test proving that contacts from all devices are removed.
- This is a source-level correction only. No build, Cargo check, rustfmt, test execution or hardware validation was performed; grouped validation remains pending.


## Touchpad interaction profile — 2026-11-25

AureaOS adopts a macOS-inspired touchpad profile while keeping the behavior native to Linux/libinput and Wayland rather than emulating macOS drivers:

- One-finger movement controls the pointer; ordinary mouse buttons and wheel events retain standard libinput semantics.
- Two-finger scrolling is handled by the existing axis path. Natural scrolling remains a per-device preference and is applied only when the hardware/driver supports it.
- Pinch and swipe sequences continue to be forwarded through the Wayland pointer-gestures protocol to applications that support them.
- Three-finger horizontal swipes switch workspaces; three-finger upward swipes open AureaOS Overview. Direction and minimum-distance checks prevent short or predominantly vertical movement from accidentally switching workspaces.
- Four-finger pinch remains an Overview shortcut as an additional gesture.
- Tap-to-click, tap-and-drag and disable-while-typing remain per-device libinput settings, with capability-aware status reporting.

The three-finger upward Overview binding is now part of `GestureProfile` and has focused unit tests for direction and threshold. Downward swipe/App Exposé, Launchpad-style gesture, configurable finger counts, gesture sensitivity UI and per-device gesture profiles are intentionally not claimed as implemented yet. They need explicit shell actions and settings contracts before being added.

Mouse handling remains standards-based: left/right/middle and additional buttons where exposed, relative/absolute motion, vertical/horizontal wheel axes, acceleration profile/speed and per-device settings. Gaming-specific DPI buttons, vendor macros or aim-assist features are not assumed; those are hardware/vendor-specific and should only appear when reliably exposed by libinput.

No build, Cargo check, rustfmt or test execution was performed for this source update; grouped validation remains pending.


## Touchpad gesture map — 2026-11-25

The default navigation gestures are:

- **Three fingers, swipe left:** move to the next virtual desktop/workspace.
- **Three fingers, swipe right:** move to the previous virtual desktop/workspace.
- **Three fingers, swipe up:** request AureaOS Overview.
- **Three fingers, swipe down:** request the App Exposé-style panel.
- **Four fingers, pinch:** request AureaOS Overview as an alternative.
- **Two fingers, scroll:** scroll content vertically or horizontally; it does not switch workspaces.

Horizontal navigation is already resolved by `GestureProfile`: a negative accumulated horizontal delta selects `NextWorkspace`, while a positive delta selects `PreviousWorkspace`. Vertical gestures are distinguished by their dominant axis and minimum movement threshold, so an ordinary diagonal or short scroll should not trigger a desktop action.

The compositor applies workspace changes directly. Overview and App Exposé are sent to the shell as semantic requests; the shell currently stores the requested panel state. The finished visual panel surfaces are a separate presentation task and are not claimed complete. User-configurable gesture counts and sensitivity remain future work.

No build, Cargo check, rustfmt or tests were run; grouped validation remains pending.


## Keyboard fallback for touchpad navigation — 2026-11-25

Desktop navigation is available without a touchpad. The global keyboard resolver uses the same semantic actions as the touchpad interaction path:

- `Super + Ctrl + Left`: previous virtual workspace.
- `Super + Ctrl + Right`: next virtual workspace.
- `Super + Ctrl + Up`: request AureaOS Overview.
- `Super + Ctrl + Down`: request App Exposé.

The compositor consumes recognized press, repeat and matching release events so these global combinations are not leaked to the focused application. Unrecognized combinations continue through the normal Wayland keyboard path. The workspace bindings call the compositor workspace switch operation; Overview and App Exposé are sent to the shell using the existing desktop-interaction channel.

This provides equivalent keyboard access for a mouse-and-keyboard setup. The visible Overview and App Exposé surfaces remain separate shell presentation work. The bindings are defaults; a user-facing shortcut customization interface is not yet implemented.

No build, Cargo check, rustfmt, test execution or hardware validation was run. Grouped validation remains pending.


## Pointing-device hot-unplug ordering and identity — 2026-11-25

- On `DeviceRemoved`, AureaOS now clears active gesture ownership, cancels any active touchscreen contacts, synthesizes releases for buttons still held by that device, removes interaction/calibration/output-routing state and clears per-device pointer/touchpad preferences before removing the device from the live inventory.
- The backend then removes the cached touchpad apply record and retained libinput handle, and finally reconciles the seat's touchscreen capability. This ordering avoids relying on an inventory entry that has already been deleted while protocol state is being cleaned up.
- Device IDs are deterministic hashes of the current kernel input sysname. They are session/runtime identifiers, not persistent physical-device identities: event-node numbering may change across reboot or reconnect. Per-device overrides are runtime state and must not be treated as durable hardware profiles.
- Focused source tests cover deterministic ID derivation for equal sysnames and distinct IDs for the current test sysnames, plus PointerManager button/preference isolation and cleanup. Hash collisions remain theoretically possible, as with any 64-bit hash; no persistent identity guarantee is made.
- No build, Cargo check, rustfmt, test execution or hardware hot-unplug/reconnect validation was run. Grouped validation remains pending.


## Pointing-device reclassification and lifecycle ownership — 2026-11-25

- Discovery now registers devices through the compositor's `PointerManager` boundary; the raw registry mutation methods are crate-private so server dispatch cannot bypass per-device preference reconciliation.
- When an existing sysname is re-announced with a different device kind, `PointerManager` reconciles overrides: mouse pointer preferences may remain valid when a touchpad becomes a mouse, touchpad-only preferences and apply reports are cleared, and pointer preferences are cleared when the device is no longer a mouse/touchpad.
- A unified compositor disconnect method first clears compositor/protocol state (including pending button releases and active contacts/gestures), then the manager removes preferences, button bookkeeping and the inventory entry. The server subsequently drops cached touchpad handles/reports and reconciles the seat touch capability.
- Added focused source tests for preference reconciliation, stale touchpad report removal and manager disconnect cleanup. These tests are not yet compiled or executed.
- No build, Cargo check, rustfmt, test execution or hardware hotplug validation was run. Grouped validation remains pending.

### Unsupported re-announcement cleanup — 2026-11-25

When libinput re-announces a sysname whose capabilities no longer classify it as a supported pointing device, the dispatch path now retires any prior pointing registration through `disconnect_pointing_device`. It also removes cached touchpad handles and applied-setting state, then refreshes seat touchscreen capability. This closes the stale-registration case where `DeviceAdded` previously produced no descriptor and therefore skipped lifecycle cleanup. A manager regression test covers disconnecting and reconnecting the same session ID and verifies that button and per-device preference state do not leak into the new connection. Source tests added; not executed in this change.
