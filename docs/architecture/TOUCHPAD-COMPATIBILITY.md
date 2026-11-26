# AureaOS Touchpad Compatibility and Feature Matrix

**Status:** research baseline and implementation plan  
**Last reviewed:** 25 November 2026  
**Platform target:** Debian-based AureaOS 1.0 Roma, Wayland + Smithay + libinput

## Objective

Support the broadest practical range of laptop-integrated and external touchpads without writing a separate AureaOS driver for every vendor. Keep hardware transport and protocol handling in the Linux kernel, normalize device capabilities through libinput, and expose consistent settings and gesture actions through AureaOS.

AureaOS must not depend on X11, XWayland, xf86-input-synaptics, GNOME, KDE, or another desktop environment for touchpad support.

## The three-layer model

1. **Kernel and transport drivers:** enumerate the device, speak its hardware protocol, and publish Linux input events.
2. **libinput:** identify touchpad capabilities, apply device-specific quirks and thresholds, normalize pointer/touch events, and recognize supported gestures.
3. **AureaOS compositor and settings:** configure the capabilities actually exposed by each device, provide a consistent settings surface, and map gestures to AureaOS actions.

Do not duplicate a vendor protocol in the compositor when the kernel already supports it. Do not promise a feature merely because another vendor's device exposes it.

## Hardware and transport families to cover

| Family / examples | Common connection or kernel path | AureaOS policy |
|---|---|---|
| Synaptics legacy | PS/2 via psmouse; older serial protocol | Support through kernel + libinput; account for limited multitouch on older models |
| Synaptics modern | RMI4 over I2C, SMBus or SPI; some HID-over-I2C devices | Support through the matching in-kernel transport/protocol driver and libinput quirks |
| ELAN | I2C or SMBus; some devices use HID paths | Support through elan_i2c or the applicable HID/I2C stack |
| ALPS | PS/2 protocol and ALPS HID/I2C variants | Support through psmouse or HID/ALPS kernel support plus libinput |
| Apple | USB (including bcm5974 family) and device-specific SPI/HID paths on some generations | Support only where the target Debian kernel provides the relevant driver; preserve vendor-specific capabilities exposed by libinput |
| Generic HID Multitouch / Precision Touchpad-compatible devices | USB, Bluetooth HID, I2C-HID and other HID transports | Prioritize as the common interoperability path for external touchpads and modern devices |
| Goodix, Cypress and other controller vendors | Commonly I2C or HID paths depending on model | Treat as capability-driven devices; verify kernel support per exact hardware rather than assume one universal vendor driver |
| External touchpads | Usually USB HID or Bluetooth HID; model-dependent | Support standard HID multitouch where the kernel exposes it; hotplug and reconnect must work |

The vendor name alone does not identify the transport or feature set. The same vendor can ship devices using different protocols, and different vendors can expose the same standardized HID interface. In particular, Goodix is widely represented in Linux as a touchscreen-controller family; a Goodix vendor/controller label must not cause AureaOS to classify a device as a touchpad without pointer/touchpad evidence.

## Relevant upstream Linux driver families

The kernel source contains separate paths for legacy and modern hardware, including `psmouse`/`synaptics`/`alps`/`elantech` for PS/2-family devices, `elan_i2c_core` and its I2C/SMBus transports, Synaptics RMI4 core with I2C/SMBus/SPI transports, generic `i2c-hid`, and HID multitouch for compatible HID devices. Apple USB touchpads have device-specific support such as `bcm5974`. These are kernel implementation families, not dependencies AureaOS should link directly; the compositor consumes their normalized libinput devices. Exact device IDs and firmware still determine whether a particular model works.

## Feature matrix

| Feature | Source / dependency | AureaOS behavior |
|---|---|---|
| Pointer movement | Kernel absolute touch data + libinput | Convert to stable relative pointer motion |
| One-, two-, three-finger tap | libinput tap configuration; hardware-dependent finger count | Configurable tap-to-click and documented click mapping |
| Tap-and-drag / drag lock | libinput tap configuration | Expose only when supported |
| Two-finger scrolling | libinput scroll methods | Enable/disable and natural-scroll direction |
| Edge scrolling | libinput scroll methods; device-dependent | Offer only when advertised by the device |
| Clickpad button areas / clickfinger | libinput click methods | Expose supported method and explain behavior |
| Swipe gestures | libinput gesture events | Route 3/4-finger swipes to configurable AureaOS actions when supported |
| Pinch and rotate | libinput pinch gesture events | Route scale/rotation deltas to supported actions |
| Three-finger drag | libinput feature/device support | Expose only when the installed libinput API and device support it |
| Palm/thumb rejection | libinput heuristics, kernel pressure/size data and device quirks | Use upstream heuristics; do not invent universal thresholds |
| Disable while typing (DWT) | libinput + keyboard integration | Expose per-device toggle when available |
| Disable while trackpointing (DWTP) | libinput + integrated pointing stick | Expose when available |
| Acceleration profile and speed | libinput | Per-device settings with supported profiles only |
| Left-handed, middle-button emulation | libinput | Expose when supported by the backend API |
| Disable/suspend touchpad events | libinput send-events modes | Provide explicit enabled/disabled/disabled-on-external-mouse modes when supported |
| Device-specific quirks | libinput hardware database | Prefer upstream quirks; report unsupported or anomalous devices for upstream fixes |

## Gesture boundary

libinput currently recognizes swipe and pinch gestures for touchpads and other indirect devices. It does not interpret touchscreen gestures. A gesture's finger count is fixed for its lifetime; hardware with only two fully tracked touch slots can impose limitations on higher-finger gesture interpretation.

AureaOS must distinguish:
- **Recognition:** the hardware, kernel and libinput can reliably report the gesture.
- **Action mapping:** AureaOS assigns a recognized gesture to an action such as workspace switching or overview.
- **Presentation:** shell animations or UI surfaces that visually respond to the action.

A missing shell surface must not be confused with missing hardware gesture support. Conversely, do not advertise a gesture action until its complete action path exists.

## Vendor and transport detection policy

AureaOS deliberately does not map a vendor ID such as Synaptics, ELAN, ALPS or Goodix directly to a fixed feature list. The same manufacturer may ship PS/2, RMI4, I2C-HID, SMBus, USB HID and Bluetooth HID products with different firmware and multitouch limits.

The kernel and libinput remain the compatibility authority:
- Kernel transport drivers enumerate the hardware and expose evdev capabilities.
- udev/libinput classify the device and apply upstream model quirks.
- AureaOS reads the resulting libinput capability/configuration surface and publishes it as a per-device snapshot.
- A setting is offered only when the relevant configuration API reports support. A missing tap interface does not disqualify a device if it exposes other touchpad-specific behavior, such as scrolling or gestures.
- No AureaOS quirk is added based only on a vendor name. Model-specific fixes must be tied to a reproducible device identity and preferably contributed upstream to libinput.

For USB and Bluetooth touchpads, the same capability-driven path is used as for internal devices. Hotplug and reconnect use the existing device lifecycle. Persistent profiles depend on the physical path available from udev/sysfs; moving a USB device to another port may create a distinct profile. If a stable path cannot be established, AureaOS skips persistence rather than guessing.

## Implementation requirements

1. Use the Debian target kernel's supported input drivers and Debian-packaged libinput. Fedora is a development host only.
2. Inventory devices using capabilities and stable hardware metadata; never classify by vendor name alone.
3. Keep optional settings capability-gated. An unsupported option must be reported as unsupported, not silently treated as applied.
4. Preserve per-device preferences across reconnects using a stable identity strategy. Do not persist Linux event node names or transient runtime IDs as device identity.
5. Ensure hotplug, removal, suspend/resume and reconnect do not retain stale handles or settings state.
6. Keep external USB/Bluetooth HID touchpads in the compatibility target, not just internal laptop touchpads.
7. Keep compositor code modular; separate capability discovery, settings application, gesture policy and persistent identity.
8. Do not add X11/XWayland-only drivers or desktop-environment services.

## Current AureaOS implementation snapshot

- libinput is integrated through Smithay.
- Touchpad detection/configuration and per-device settings/reporting exist.
- Touchpad detection uses libinput capabilities rather than vendor-name heuristics and recognizes scroll-only and clickpad-only devices even when tap, DWT and gesture capability are absent.
- The shared Desktop API now exposes tap-to-click, tap-and-drag, drag lock, tap-button mapping, two-finger/edge scrolling, natural scrolling, disable-while-typing, clickpad method, left-handed mode, middle-button emulation, pointer acceleration profile/speed and send-events mode.
- The compositor applies each option only when the installed Smithay/libinput device reports that capability. Every option returns an explicit Applied, Unsupported or Failed status.
- Device discovery now publishes a detailed TouchpadCapabilities snapshot through the shared Desktop API: maximum tap-finger count, two-finger/edge scrolling, natural-scroll support, click methods, DWT/DWTP, gestures, left-handed mode, middle-button emulation and acceleration profiles. The Settings UI can therefore hide or disable controls based on the actual connected device rather than vendor guesses.
- The existing five-argument TouchpadSettings constructor remains source-compatible; advanced options use builder-style methods.
- Clickpad method, tap mapping, drag-lock, acceleration profile/speed and send-events configuration are implemented against the pinned Smithay 0.7.0 / input 0.9.1 API. The send-events API uses input 0.9.1 bitflags (not the later input 0.10 enum/vector signatures); this was checked and corrected against the exact crate source.
- Disable-while-trackpointing (DWTP) is now wired when the native `drm` backend is built. Smithay 0.7.0 resolves `input` 0.9.1; the compositor adds that exact crate as an optional direct dependency and enables `input/libinput_1_21` only with the native backend. Cargo feature unification exposes the DWTP methods on the same `input` crate instance re-exported by Smithay. Devices without the capability report Unsupported.
- The gesture lifecycle has now been source-audited: swipe, pinch and hold begin/update/end events are forwarded to Wayland pointer-gesture clients; cancelled sequences do not trigger AureaOS actions; per-device active sequences and queued actions are cleared on device removal. The Wayland seat stream is serialized to one owner at a time.
- Persistent touchpad preferences are stored by a SHA-256 identity derived from vendor/product IDs and udev `ID_PATH`; when that property is unavailable, the normalized physical sysfs path is used. Kernel event-node numbers, display names and serial numbers are excluded. If neither path is available, persistence is skipped rather than falling back to a collision-prone identity.
- Profiles are saved atomically to `$XDG_CONFIG_HOME/aureaos/input/touchpads.toml` or `~/.config/aureaos/input/touchpads.toml`, with private file permissions. Profiles are restored when the matching device reconnects. External USB devices moved to another physical port may receive a different identity because serial numbers are deliberately not collected.
- Smithay is pinned to crates.io release `=0.7.0`; Cargo downloads it and its transitive crates normally. No Smithay source is copied into AureaOS and no fork is currently necessary. The manifest dependency additions are in place, but `Cargo.lock` still needs its grouped refresh on Fedora because no Cargo operation has been run in this pass.
- Debian 13 (trixie) provides libinput 1.28.1 and Debian 12 (bookworm) provides 1.22.1, both above the 1.21 API floor required for DWTP. Older systems below 1.21 are outside this native backend's current build baseline. <https://packages.debian.org/trixie/libinput-dev> <https://packages.debian.org/bookworm/libinput-dev>
- No build, Cargo check, rustfmt, unit tests or physical-hardware validation has been run yet; validation remains deliberately grouped.

## Next implementation sequence

1. Finish the code-level audit of gesture sequencing, cancellation, ownership and action dispatch; keep Wayland client gesture forwarding separate from AureaOS shell actions.
2. Expose and consume the live touchpad capability and apply-report snapshots in the Settings UI, hiding or disabling unsupported controls.
3. Verify stable identity behavior on real I2C, PS/2, USB and Bluetooth devices, especially USB port changes and composite input devices.
4. Verify Debian target kernel driver coverage for representative Synaptics, ELAN, ALPS, Apple and generic HID devices.
5. Test internal and external USB/Bluetooth HID touchpads, hotplug, suspend/resume and reconnect.
6. Perform grouped formatting, build and unit-test validation after the pointing-device work is complete, followed by physical hardware validation.

## Research references

- Linux Touchpad Dev Guide, input stack: https://linuxtouchpad.org/docs/
- libinput configuration: https://wayland.freedesktop.org/libinput/doc/latest/configuration.html
- libinput gestures: https://wayland.freedesktop.org/libinput/doc/latest/gestures.html
- libinput features: https://wayland.freedesktop.org/libinput/doc/latest/features.html
- libinput device quirks: https://wayland.freedesktop.org/libinput/doc/latest/device-quirks.html
- Linux kernel HID and HID multitouch drivers: https://code.googlesource.com/linux/torvalds/linux/+/master/drivers/hid/
- Linux kernel mouse/input drivers: https://code.googlesource.com/linux/torvalds/linux/+/master/drivers/input/mouse/

Research note: vendor and transport coverage varies by exact device ID, kernel version and firmware. The matrix is a compatibility target, not a claim that every device model has been hardware-tested.


## Capability snapshot API — 2026-11-25

The shared Desktop API now carries optional TouchpadCapabilities metadata alongside the standard input capability list. The compositor fills it only for devices classified as touchpads. It reports maximum tap-finger count, supported scroll/click methods, natural-scroll availability, DWT/DWTP, gesture support, handedness, middle-button emulation and acceleration profiles. This snapshot is a live observation from the installed kernel/libinput stack; it is not a vendor catalogue and is not persisted as a hardware promise.

The snapshot is intentionally independent of the settings values. It lets Aurea Settings decide which controls to show while the existing TouchpadApplyReport continues to report whether a requested value was actually applied.

Source changes have been committed but not compiled or tested. Do not treat the API integration as build-verified until the grouped Fedora validation.


Kernel source references used for the driver-family audit: Linux input/mouse driver tree (including ALPS, ELAN I2C, Synaptics, psmouse and bcm5974): https://kernel.googlesource.com/pub/scm/linux/kernel/git/sailus/linux/+/refs/heads/for-next/drivers/input/mouse/ . The driver inventory is upstream-source evidence, not a guarantee that every driver is enabled in a given Debian kernel configuration.


## Detailed tap and device-disable capabilities

The per-device capability snapshot also reports whether tap-to-click controls are
available (maximum tap finger count greater than zero), including tap-and-drag,
drag lock and tap button mapping. These are configuration surfaces exposed by
libinput for tapping devices; they are not inferred from a manufacturer name.

The snapshot records whether libinput offers the two optional send-events modes:
disable the device, and disable it while an external mouse is present. The
always-available enabled mode is not represented as an optional capability.
This keeps the settings UI aligned with the modes the active kernel/udev/libinput
stack actually exposes. The values are capability indicators, not the current
enabled state.

The Rust `input 0.9.1` API used by AureaOS exposes the tap getters/setters and
send-events mode bitflags. Three-finger drag is intentionally not advertised
yet: it is not present in the pinned `input 0.9.1` Device API, and must not be
claimed based on newer libinput documentation alone.


## Session pause and resume

On a libseat/VT pause, AureaOS stops forwarding queued input. On activation,
the compositor now invalidates its cache of applied touchpad profiles and
reapplies each saved profile to the retained live libinput device handles.
This is deliberately separate from device discovery: a VT switch is not
treated as a physical unplug/replug, so stable per-device identity and saved
preferences remain intact.

If libinput reports an individual setting as unsupported or failed, the
per-setting application report remains visible and the profile is not marked
fully applied. A later input event can retry through the normal refresh path.

## Shake-to-locate and pointer trail

The macOS-style locate gesture is available on touchpads through the same libinput relative pointer-motion events used for ordinary cursor movement. AureaOS feeds unaccelerated relative deltas into the per-device shake detector rather than treating a Wayland swipe/pinch gesture as a locate command. The detector looks for rapid alternating movement; users do not need to trace a circle.

The compositor implements temporary pointer enlargement and a separate fading trail for the native cursor. The trail is a rendering effect, not a libinput feature; it is drawn behind the compositor-owned cursor and does not apply to client-provided Wayland cursor surfaces. Physical tests across different touchpad vendors/drivers are still required.


## Follow-up audit — 26 November 2026

The DeviceAdded path now registers the discovered device before applying touchpad preferences. Registration restores the profile keyed by the stable physical identity; the compositor then configures libinput from that restored profile. This avoids first applying defaults and waiting for the first input event to refresh the device to its saved preferences.

The runtime audit confirms that touchpad settings are capability-gated and each setting reports Applied, Unsupported or Failed. Device removal clears transient per-device state and active gesture ownership; persisted profiles remain available for reconnection. Session activation invalidates the applied-settings cache and reapplies profiles to retained live handles.

Still outstanding: grouped compile/format/test validation, physical hardware testing, and verification of the Settings UI read/write path. Gesture recognition and shell action execution must also be distinguished: libinput events are forwarded to Wayland pointer-gesture clients, while completed AureaOS desktop actions are dispatched through the compositor-to-shell interaction channel. No claim of hardware validation is made.


## Pointer and touchpad source audit — 26 November 2026

The focused source audit covered libinput discovery/dispatch, relative pointer motion, absolute-pointer routing, button aggregation, axis frames, touchpad configuration, per-device settings, gesture ownership, device removal/reclassification and libseat pause/resume.

A concrete lifecycle gap was found and fixed: a VT/libseat pause released held pointer buttons but did not terminate an in-progress swipe, pinch or hold sequence. If libinput did not deliver the matching end event while the session was inactive, the gesture owner and semantic accumulator could survive the pause and accept a later update/end after activation. The compositor now takes and clears the active gesture owner, marks the matching semantic sequence cancelled, sends the corresponding cancelled Wayland pointer-gesture end event, and frames the pointer before the session is paused. A unit test covers clearing the owner and allowing a new sequence. This is a source change; it has not yet been compiled or executed.

Other source-audit findings:
- Relative mouse/touchpad motion is finite-value checked, constrained to enabled output rectangles, committed through one shared focus/frame path and requests redraws on all outputs. The pointer therefore crosses adjoining/overlapping display layouts without being stranded in gaps.
- Absolute pointer motion is intentionally separate. It routes automatically only on a single enabled output; multi-output absolute devices require an explicit output assignment. This is not a limitation for ordinary relative mice/touchpads.
- Physical button holds are tracked per device and aggregated at the shared Wayland seat. Duplicate transitions are suppressed, auxiliary buttons are passed through, and disconnect/session pause synthesize releases.
- Scroll frames preserve horizontal/vertical continuous values, v120 high-resolution values, source and relative direction, while rejecting non-finite or unrepresentable values.
- Touchpad configuration is applied capability-by-capability and reports Applied, Unsupported or Failed. Saved profiles are restored on discovery and session activation; mouse acceleration is likewise reapplied on activation.
- Device removal and device-class changes clear device-scoped settings, held buttons, gesture ownership and transient pointing state.

Scope decision: Wacom pen/tablet and Tablet Pad remain deferred as previously agreed. Settings-page presentation of pointer speed, double-click interval and touchpad controls is a separate shell/UI task; the compositor-side settings and persistence contracts are present. Overview/App Exposé visual presentation is also a desktop renderer task, not a missing mouse/touchpad event route.

Remaining before declaring the block operationally verified: grouped rustfmt/build/tests on Fedora, then physical testing with a mouse and at least one touchpad (including tap/click, scrolling, gestures, unplug/reconnect and VT pause/resume). The source audit does not establish runtime or hardware success.


### Gesture cancellation on physical removal — 26 November 2026

A second lifecycle gap was found during the follow-up audit. When the touchpad owning an active swipe, pinch or hold was physically removed, AureaOS cleared its internal owner and semantic state but did not send the matching cancelled end event to Wayland pointer-gesture clients. A client could therefore retain an unfinished protocol gesture.

The disconnect path now takes the active gesture only when the removed device owns it, cancels the semantic sequence, sends the corresponding Wayland end event with `cancelled = true`, and frames the pointer before unregistering the device. Other devices' gesture ownership is not disturbed. The owner also rejects duplicate begin events instead of resetting an already active sequence.

Added unit coverage for duplicate begin rejection and device-scoped owner removal. Source changes are committed but have not been compiled or executed; grouped validation remains pending.

## Gesture begin guards and owner-test consistency — 26 November 2026

A follow-up source audit found that the swipe, pinch, and hold begin handlers
claimed gesture ownership and initialized semantic state before confirming that
the Wayland seat exposed a pointer. The handlers now acquire the pointer first,
so an unavailable pointer cannot leave an orphaned gesture owner or accumulator.

The same audit found one owner unit test still calling the removed
`remove_device` helper. It now verifies the current `take_device` contract and
checks that the removed device's gesture kind is returned before another device
claims the stream.

These are source changes only. The grouped Rust formatting, compile, and test
pass remains pending, as requested; no hardware validation has been performed.

## Accurate no-scroll result reporting — 26 November 2026

The scroll configuration audit found that the `NoScroll` branch marked both
two-finger and edge scrolling as `Applied` even if libinput rejected the
requested `NoScroll` method. The report now propagates the actual result of
`config_scroll_set_method(NoScroll)` to both settings, so an unsupported or
failed operation is not presented as successful.

This follows libinput's contract: `get_methods` reports the methods supported
by the device and `set_method` returns a configuration status. Source:
https://wayland.freedesktop.org/libinput/doc/latest/api/group__config.html

This is a source-level correction; grouped build and tests remain pending.

## Separate mouse and touchpad acceleration ownership — 26 November 2026

The audit found two competing configuration paths for touchpad acceleration:
the touchpad profile applied its own speed and acceleration curve, while the
generic pointer profile was also applied on device discovery, pointer motion,
and session activation. The generic path could therefore overwrite the
touchpad-specific preference immediately or on the next motion event.

Touchpad speed/profile now have one owner: `TouchpadSettings`, including
discovery, live updates, motion refresh, and session reactivation. Generic
`PointerSettings` are restricted to mice; their persisted profiles are no
longer restored to touchpads, and the mouse acceleration adapter is no longer
run against touchpad handles. Regression assertions reject assigning a mouse
profile to a touchpad.

No build or test run was performed; the grouped validation remains pending.


## Reporting mutuamente esclusivo delle modalità di scorrimento

Il report touchpad distingue tre casi: `Applied` quando la modalità richiesta è stata impostata, `Unsupported` quando il dispositivo non espone la capacità e `NotRequested` quando l'opzione è inattiva perché è stata scelta la modalità alternativa. Lo scorrimento a due dita e quello ai bordi sono mutuamente esclusivi: il report non deve presentare entrambe le modalità come applicate. `all_applied()` ignora `NotRequested` ma non `Unsupported` o `Failed`.


### Modalità senza scorrimento

Quando l'utente sceglie esplicitamente `NoScroll`, entrambe le modalità di scorrimento (a due dita e dai bordi) sono `NotRequested`. Non vanno riportate come `Applied`: il successo riguarda la scelta di non usare quei gesti, non l'applicazione di una delle due modalità.

## Simplification rule — reuse the Linux input stack

AureaOS does not implement hardware protocols, pointer acceleration, touchpad filtering,
palm rejection, gesture recognition or external-mouse detection when those functions
are already provided by the Debian kernel and libinput. The compositor consumes the
normalized events and capability/configuration API; AureaOS owns only the desktop
policy, gesture-to-action mapping and user-facing preferences.

In particular:
- Kernel drivers and evdev remain responsible for hardware transport.
- udev/libinput remain responsible for device quirks, classification, normalization
  and supported touchpad gesture recognition.
- Smithay remains the Wayland compositor integration layer.
- AureaOS maps supported gestures to workspace/overview actions and exposes only
  settings advertised by the connected device.
- DisabledOnExternalMouse is delegated to libinput; AureaOS must not add a parallel
  external-mouse detector.
- DeviceDefault means restore libinput's own per-device default through its default
  getter/setter APIs when available. If the binding exposes no default value, the
  report marks the option NotRequested rather than inventing a policy.

Do not add a new service, registry or capability abstraction unless a concrete
compositor or shell consumer requires it. Keep the input lifecycle state needed
for Wayland correctness (for example, button release and gesture cancellation),
but do not duplicate libinput's device-processing logic.

The mouse/touchpad backend is limited to this integration boundary. Settings UI
and shell presentation are separate product work; Wacom/tablet support remains
deferred.
