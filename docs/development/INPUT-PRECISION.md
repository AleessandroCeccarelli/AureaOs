# AureaOS Input Precision and Professional Peripherals

Status: implementation in progress  
Target: AureaOS 1.0 Roma  
Backend: libinput through Smithay; Wayland seat delivery

## Design contract

AureaOS must support ordinary desktop pointing devices and professional digitizers without applying one device class's policy to another.

### Relative mouse and touchpad motion

- Adaptive/flat acceleration and pointer speed are policies for relative mouse and touchpad devices only.
- Preserve the backend's floating-point deltas. Do not round to integer pixels in the input path.
- Keep accelerated and unaccelerated deltas distinct. The latter is used for shake-to-locate so user pointer acceleration does not distort motion detection.
- Reject non-finite samples before they can contaminate the shared seat cursor coordinates.
- Do not synchronously print or format diagnostics for every Nth motion sample in the high-frequency event path. Such writes can block the event loop and introduce jitter on high-polling-rate devices.
- Do not arbitrarily coalesce or throttle input samples. Any future batching must preserve event ordering, timestamps, button transitions and tablet axis changes.

### Scroll wheels and high-resolution devices

- Preserve horizontal and vertical scroll independently.
- Forward both continuous axis values and libinput's v120 values to the Wayland pointer axis frame.
- Preserve axis source and relative direction.
- Never convert high-resolution wheel input to coarse detents in the compositor. Applications receive the Wayland axis information and decide how to consume it.
- Device support must be advertised only from verified backend capability data; do not infer it from a marketing name such as MX Master.

### Pen tablets and digitizers

- Tablet-tool events use the dedicated Smithay tablet protocol path, not mouse acceleration.
- Preserve absolute coordinates, pressure, distance, tilt, rotation, slider/wheel values and tool button transitions when the backend reports changes.
- Keep pen tip and barrel-button state associated with the originating physical tablet; close proximity and synthesize releases on disconnect.
- Keep per-tablet output assignment and calibration independent. A disconnected display must not silently redirect a tablet to another monitor.
- The current code has per-device calibration/output mapping, a tablet-tool ownership registry and durable physical-device keyed calibration profiles. Persisting tablet-to-monitor assignment is intentionally separate because runtime output IDs are not stable physical monitor identities.
- Verify eraser and per-tool capabilities from the actual tablet tool descriptor before exposing them in Settings; do not guess from vendor/product strings.

### Multiple devices

- Maintain independent device identity, preferences and physical button state.
- Aggregate duplicate physical button holds at the shared Wayland seat: only the first press and final release should cross the seat boundary.
- Reconnection must restore preferences only when a reliable physical identity is available. Session IDs and eventN names are not persistent identities.

## Work completed in this block

- Removed synchronous stderr logging from relative motion, pointer-button and scroll-event hot paths in the compositor/libinput dispatch. Device discovery/removal diagnostics remain event-scoped.
- Added finite-value validation for accelerated and unaccelerated motion samples. Valid sub-pixel floating-point values remain untouched.
- Made the dispatch path explicitly apply pointer acceleration only to registered mouse and touchpad devices. Tablet and touchscreen classes cannot inherit this policy accidentally.
- Made pointer and touchpad profile stores roll back their in-memory change when persistence fails, so the live profile cache cannot diverge from the on-disk profile. Added assertions for this failure path.
- Confirmed the existing scroll dispatch passes horizontal/vertical continuous values, horizontal/vertical v120 values, source and relative direction to the compositor's axis-frame implementation.
- Confirmed the tablet path already forwards pressure, distance, tilt, rotation, slider and wheel changes through Smithay's tablet tool handle.

## Verification still required

No build, formatter, unit test, hardware test or benchmark was run for this block. Run these together on the Fedora development machine as requested. The source-level work is still under the pointer/input block; do not treat this as permission to move on to unrelated desktop features.

Hardware validation matrix:
1. Logitech MX Master 4 (including high-resolution horizontal/vertical wheel behavior and side buttons).
2. Standard USB/Bluetooth mouse at more than one polling rate.
3. Wacom pen with pressure, tilt, rotation, eraser and barrel buttons where the specific model supports them.
4. Tablet plus mouse plus touchpad connected concurrently.
5. Tablet mapped to one monitor in a multi-monitor layout, including disconnect/reconnect of the assigned monitor.
6. Disconnect/reconnect of each peripheral while buttons or pen proximity are active.

Do not claim a particular polling rate, DPI, button, pen axis or feature is supported until libinput/device capability data or physical testing confirms it.
