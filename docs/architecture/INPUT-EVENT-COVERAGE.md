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
| Absolute pointer motion | Diagnostic trace only; no output mapping policy | Not implemented |
| Switch toggle | State-change diagnostic only; no lid/tablet-mode policy | Not implemented |
| Tablet pad controls | No corresponding Smithay 0.7 InputEvent path | Not available through this event API |
| Special backend events | No AureaOS policy | Ignored |

## Source changes in this audit

- Added explicit `PointerMotionAbsolute` handling in `server/input/dispatch.rs`. It records device identity and raw coordinates at debug level. It deliberately does not move the pointer: absolute devices need an explicit output assignment/mapping policy first.
- Added explicit `SwitchToggle` handling. It records device, switch and state. Power/session policy for lid close and tablet mode remains outside the compositor input router.
- Kept the final wildcard for future/non-actionable backend variants; the known absolute and switch cases are no longer silently swallowed.

## Follow-up implementation sequence

1. Define and expose an absolute-pointer output mapping contract before converting absolute coordinates into global logical coordinates.
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

When a tablet is mapped to the full desktop, the rectangular calibration extent can span empty gaps between non-adjacent or offset monitors. Tablet coordinates are now constrained against the actual enabled output rectangles after calibration. A point falling in a gap is moved to the nearest valid output edge, following the same geometry policy already used by relative pointer motion. For a tablet explicitly mapped to one output, the candidate geometry is restricted to that output, so the pen cannot escape its assigned display. This does not change touchscreen transforms or absolute-pointer routing.
