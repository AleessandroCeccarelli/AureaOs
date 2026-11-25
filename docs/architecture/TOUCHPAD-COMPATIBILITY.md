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

The vendor name alone does not identify the transport or feature set. The same vendor can ship devices using different protocols, and different vendors can expose the same standardized HID interface.

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
- The existing five-argument TouchpadSettings constructor remains source-compatible; advanced options use builder-style methods.
- Clickpad method, tap mapping, drag-lock, acceleration profile/speed and send-events configuration are implemented against the pinned Smithay 0.7.0 / input 0.9.1 API. The send-events API uses input 0.9.1 bitflags (not the later input 0.10 enum/vector signatures); this was checked and corrected against the exact crate source.
- Disable-while-trackpointing (DWTP) is documented as a desired capability but is not yet wired: Smithay 0.7.0 enables input's libinput_1_19 feature, while the DWTP methods in input 0.9.1 are gated behind libinput_1_21. Do not call these methods until the dependency feature strategy and Debian minimum libinput version are deliberately resolved.
- The gesture lifecycle has now been source-audited: swipe, pinch and hold begin/update/end events are forwarded to Wayland pointer-gesture clients; cancelled sequences do not trigger AureaOS actions; per-device active sequences and queued actions are cleared on device removal. The Wayland seat stream is serialized to one owner at a time.
- Persistent stable hardware identity and durable per-device preference storage remain future work.
- No build, Cargo check, rustfmt, unit tests or physical-hardware validation has been run yet; validation remains deliberately grouped.

## Next implementation sequence

1. Audit gesture begin/update/end/cancel handling and map only gestures actually emitted by libinput, including cancellation and device removal.
2. Add durable per-device preferences keyed by stable physical identity (vendor/product plus libinput device group/udev identity as available), not event node or transient runtime ID.
3. Extend capability reporting to include gesture counts and device metadata needed by the Settings UI.
4. Verify Debian target kernel/libinput versions and their input-driver coverage for representative Synaptics, ELAN, ALPS, Apple and generic HID devices.
5. Test internal and external USB/Bluetooth HID touchpads, hotplug, suspend/resume and reconnect.
6. Perform grouped build, formatting, unit tests and hardware validation after the pointing-device work is complete.

## Research references

- Linux Touchpad Dev Guide, input stack: https://linuxtouchpad.org/docs/
- libinput configuration: https://wayland.freedesktop.org/libinput/doc/latest/configuration.html
- libinput gestures: https://wayland.freedesktop.org/libinput/doc/latest/gestures.html
- libinput features: https://wayland.freedesktop.org/libinput/doc/latest/features.html
- libinput device quirks: https://wayland.freedesktop.org/libinput/doc/latest/device-quirks.html
- Linux kernel HID and HID multitouch drivers: https://code.googlesource.com/linux/torvalds/linux/+/master/drivers/hid/
- Linux kernel mouse/input drivers: https://code.googlesource.com/linux/torvalds/linux/+/master/drivers/input/mouse/

Research note: vendor and transport coverage varies by exact device ID, kernel version and firmware. The matrix is a compatibility target, not a claim that every device model has been hardware-tested.
