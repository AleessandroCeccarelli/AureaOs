<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS — Keyboard Hardware and Special Input Policy

## Purpose

AureaOS must support ordinary keyboards and keyboards that expose dedicated system or media controls. This policy is part of the desktop input foundation, not a late shell enhancement.

The target includes desktop USB keyboards, laptop built-in keyboards, Bluetooth keyboards, compact layouts, international layouts and devices with vendor-specific keys. Hardware capabilities vary; AureaOS must detect and handle what a device actually exposes rather than assume every keyboard has the same keys.

## Input classes

The input architecture must distinguish these classes:

1. **Text and navigation keys:** printable keys, modifiers, function keys, navigation cluster and keypad.
2. **Desktop shortcuts:** deliberate key combinations owned by the shell or compositor.
3. **Media controls:** play/pause, stop, previous track, next track and other standard transport controls.
4. **Audio controls:** volume up, volume down and mute.
5. **Display controls:** screen brightness up/down and, where exposed, display switching or related controls.
6. **Keyboard hardware controls:** keyboard backlight toggle/up/down and device-specific illumination controls.
7. **Text and composition controls:** dedicated emoji key, compose/input-method keys and keys that invoke text-entry assistance.
8. **Vendor-specific controls:** extra keys exposed by a device which do not have a portable standard meaning.

A laptop's Fn key is not guaranteed to be visible to the operating system. Firmware may translate Fn combinations into standard key events, expose a vendor event, or handle the operation internally. AureaOS must not assume that Fn itself can always be intercepted.

## Architecture boundaries

Keep hardware input separate from command execution:

- **Backend/device layer:** discovers keyboards and receives raw evdev/libinput events, including device identity and hotplug/removal.
- **Key normalization layer:** maps supported hardware events into stable AureaOS semantic input actions. It must preserve the distinction between physical key events, XKB keysyms and layout-dependent text.
- **Keyboard policy and shortcut resolver:** decides whether an event belongs to the focused Wayland client, a global shortcut, or a system-control action. Application shortcuts remain with the focused client.
- **Desktop command coordinator:** dispatches semantic actions to the appropriate service. It must not implement audio, display, media-session or text-input functionality itself.
- **System services:** own the actual operation: audio service for volume/mute, display/power service for panel brightness, keyboard-device service for keyboard illumination, media-session routing for transport controls, and shell/text-input integration for the emoji picker.
- **Shell presentation:** provides user-visible feedback and settings for supported controls without making the compositor depend on UI implementation details.

Keep each layer small and single-purpose. Do not place hardware-specific tables, service operations and keyboard event filtering in one large keyboard handler.

## Expected behavior

| Hardware action | Semantic destination |
|---|---|
| Volume up / down / mute | Audio service |
| Play / pause / stop | Active media-session service |
| Previous / next track | Active media-session service |
| Screen brightness up / down | Display or power service |
| Keyboard backlight toggle / up / down | Keyboard-device service, when supported |
| Dedicated emoji key | Shell emoji picker, when text input permits |
| Layout and compose keys | XKB/input-method path |
| Unknown or unsupported special key | Safe fallback; do not invent an action |

A system action must not be advertised as working until its destination service exists and the end-to-end behavior is tested. If no suitable device or service is available, fail safely and avoid swallowing unrelated input.

Brightness of the display and brightness of the keyboard are separate controls and must never be conflated.

## Device and layout compatibility

- Use the Linux input stack for device discovery and hardware events; use XKB for keyboard layouts, key symbols and modifier interpretation.
- Support device hotplug and removal without leaving stuck keys, modifiers or consumed shortcut state.
- Do not assume a US layout or hard-code printable characters from physical key positions.
- Keep physical key identity, XKB keysym and resulting text as distinct concepts.
- Handle key press, release and repeat consistently. A consumed press must have a coherent release path; repeat must not repeatedly trigger one-shot desktop actions.
- Support multiple simultaneously connected keyboards and track per-device state where required.
- Treat vendor-specific keys as optional capabilities. Unknown events must not crash the compositor or be misinterpreted as ordinary text.
- Do not promise support for a hardware feature that firmware handles internally and does not expose to the operating system.

## Configuration and user experience

The future keyboard settings area should expose, as the corresponding services become available:

- active layout and input sources;
- layout switching and compose/input-method behavior;
- repeat delay and repeat rate;
- shortcut reference and, later, safe user remapping;
- supported special-key behavior;
- keyboard illumination controls only when the hardware exposes them.

The emoji key should invoke the AureaOS emoji picker through a shell command, not inject an arbitrary character or depend on a particular keyboard vendor.

## Validation requirements

Validation must cover both resolver logic and service integration:

- ordinary typing, modifiers, navigation and keypad;
- at least two different XKB layouts, including non-US behavior;
- media, volume and brightness events using synthetic input events;
- keyboard backlight events separately from display brightness;
- dedicated emoji action and behavior when text input is unavailable;
- press/release/repeat handling, including held keys;
- two keyboards connected at once, hotplug and device removal;
- unknown and vendor-specific events;
- missing or unavailable target services;
- no accidental interception of application-level shortcuts.

Hardware testing on representative physical devices is still required. Synthetic tests prove event handling, not that every laptop firmware or keyboard exposes a given feature.

## Implementation rule

This document defines the compatibility target; it does not claim that these features are already implemented. Introduce semantic actions and service contracts incrementally, only when the receiving service is real. Keep the keyboard event path focused on input routing and never turn it into a collection of direct hardware-control operations.
