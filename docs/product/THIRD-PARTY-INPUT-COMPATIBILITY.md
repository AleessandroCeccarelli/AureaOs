<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS — Third-Party Input Device Compatibility

## Purpose

AureaOS targets the continuity users expect from modern Windows and macOS desktops when they connect third-party keyboards, mice and productivity peripherals.

The design target is the user experience and compatibility model offered by those platforms, not their internal implementation and not the limitations of any other operating system. This document is a product and architecture policy; it does not claim that the described capabilities are already implemented.

## 1. Findings from Windows, macOS and Logitech

### 1.1 Operating-system baseline

Windows documents modern keyboard input through HID Usage values. Its input stack distinguishes keyboard/keypad usages from Consumer Control usages, including standard media and audio controls such as next track, previous track, stop, play/pause, mute and volume increment/decrement. Windows also exposes distinct top-level HID collections for keyboards, system controls and consumer controls.

macOS provides native handling for common function-row system actions. Users can choose whether the top row acts as system controls or standard F1–F12 keys, with Fn/Globe modifying the behavior. The available behavior depends on the keyboard and the functions it exposes.

Product implication: common standardized functions should work without requiring a vendor utility. The system must understand semantic hardware controls rather than treating every key as ordinary text.

### 1.2 Manufacturer enhancement layer

Logitech states that some advanced keys use native operating-system key codes and work without its customization software, while newer or vendor-specific actions may not have an existing standard code. Logitech explicitly cites Emoji, Dictation and video-call mute as examples requiring Logi Options+ for their intended behavior on Windows and macOS.

Logi Options+ adds capabilities beyond basic input:
- per-button customization;
- per-application profiles;
- Smart Actions and automation;
- gesture configuration and application-specific behavior;
- cross-computer Flow experiences;
- device-specific configuration and firmware-related workflows.

Product implication: OS-level compatibility and vendor-level enhancement are separate layers. AureaOS must provide a strong generic baseline and a safe extension path for richer device-specific behavior.

### 1.3 Examples from MX devices

MX Keys S advertises a multi-OS keyboard, Easy-Switch, smart backlighting and dedicated Emoji, Dictation and mute/unmute keys. Logitech documents that some customization and advanced actions require Options+.

MX Master 4 adds configurable buttons and gestures, Actions Ring and haptic feedback. Logitech also documents native Windows 11 haptic integration for selected operating-system actions, with firmware and OS-version requirements.

Product implication: compatibility includes more than key presses. It can include device configuration, app-aware actions, firmware capabilities, feedback and coordinated experiences across multiple peripherals. These advanced capabilities must be introduced only through documented, permissioned and maintainable integrations.

## 2. AureaOS compatibility model

AureaOS adopts a layered model inspired by the user-visible outcomes of Windows and macOS.

### Layer A — Universal plug-and-play

The desktop must support standardized keyboard and pointing-device behavior without requiring a vendor-specific application:
- ordinary typing, modifiers, navigation and function keys;
- standard media transport controls;
- volume and mute;
- standard display brightness controls when exposed by the device and supported display hardware;
- standard mouse buttons, wheel and horizontal scrolling;
- device connection, disconnection and reconnection;
- multiple keyboards and pointing devices.

A capability must only be shown as available when the connected device and relevant system service actually expose it.

### Layer B — Consistent OS behavior

AureaOS owns stable system semantics for common controls:
- media keys route to the active media session;
- audio keys route to the audio service;
- display brightness routes to display/power management;
- keyboard illumination routes to a keyboard-device capability;
- Emoji invokes the AureaOS emoji picker;
- Dictation invokes an available speech-input service;
- application navigation keys are delivered to the focused application unless the user has explicitly configured a global action.

The same semantic action should behave consistently across device brands. User-visible labels and settings must describe the action, not expose raw hardware codes.

### Layer C — Optional manufacturer extensions

AureaOS should provide an extension boundary for advanced vendor capabilities without making the core desktop dependent on one manufacturer.

Potential extension capabilities include:
- custom button and key assignments;
- per-application profiles;
- gestures and multi-action controls;
- device lighting and backlight configuration;
- battery and connection status;
- device firmware update workflows, only through a safe and supported mechanism;
- advanced haptic feedback;
- multi-device handoff or coordinated peripheral workflows.

Extensions must be optional, isolated, permissioned, versioned and incapable of destabilizing the compositor. A missing extension must never break standard keyboard or mouse operation.

Do not claim Logitech Options+ compatibility or reproduce its proprietary protocols without a supported interface and independent technical validation. Start with public standards and documented integration points. Vendor-specific support can be evaluated separately.

## 3. Hardware identity and capability discovery

The system must distinguish:
- physical device identity and connection transport;
- device model/vendor identity where available;
- standard HID usages and report capabilities;
- XKB key symbols and active keyboard layout;
- semantic AureaOS actions;
- optional vendor-specific capabilities.

Never infer a capability solely from a marketing name or a vendor/product identifier. Prefer actual reports, descriptors, documented capability declarations and successful service probing.

The discovery system must handle USB, Bluetooth and supported wireless receivers, device hotplug, multiple devices, battery state where exposed, and clean removal. Per-device state must be cleared on disconnect so held keys, modifiers or buttons cannot remain stuck.

## 4. Keyboard behavior and layout continuity

- Preserve the distinction between physical key identity, logical key symbol and resulting text.
- Respect the active layout; do not hard-code US character positions.
- Provide a user-selectable function-row policy where the hardware exposes both media/system and F-key behavior.
- Support Fn lock only when the device or firmware exposes a usable state or event; do not pretend the Fn key is always visible to the OS.
- Do not intercept application-level shortcuts unnecessarily.
- Keep key press, release and repeat behavior coherent.
- Unknown vendor keys must be safely ignored or exposed as unassigned inputs; never reinterpret them as arbitrary text.
- Dedicated Emoji and Dictation controls must invoke system services through semantic commands, not depend on a Logitech-specific key code.

## 5. Mouse and advanced pointing-device continuity

The same model applies to third-party mice and productivity controllers:
- baseline pointer movement, buttons, wheel and horizontal scrolling must work without vendor software;
- standard back/forward buttons should have consistent navigation behavior where appropriate;
- additional buttons must be discoverable and safely configurable;
- gesture controls, app-specific profiles, action overlays and haptics belong to optional shell/device integration layers;
- high-resolution sensor modes, firmware configuration and proprietary controls must not be promised unless a supported integration exists.

AureaOS must keep device support separate from the shell's visual design. A future AureaOS device-settings experience can present supported capabilities consistently across brands.

## 6. Security and reliability

- Do not run vendor device code inside the compositor process.
- Do not grant a peripheral unrestricted access to the desktop or user data.
- Require explicit user approval for extensions that execute commands, inspect application context, transfer data or modify system settings.
- Validate all device-originated reports and extension messages.
- Rate-limit repeated actions where appropriate and prevent stuck or runaway input.
- Keep standard input functional if an extension crashes, is removed or is unavailable.
- Firmware updates must be explicit, recoverable and handled outside the compositor.
- Log device and capability failures without collecting typed text or sensitive input.

## 7. Architecture boundaries

Keep responsibilities separated into small modules and services:

- **Device discovery manager:** device lifecycle, identity and hotplug.
- **HID capability reader:** descriptors, standard usages and supported reports.
- **Input normalization:** convert hardware reports into stable semantic events.
- **Keyboard layout manager:** XKB layouts, modifiers, key symbols and input methods.
- **Shortcut resolver:** global desktop shortcut policy, separate from application shortcuts.
- **Input command coordinator:** dispatch semantic actions without implementing service behavior.
- **Audio service:** volume, mute and audio routing.
- **Media session service:** play/pause, stop and track navigation.
- **Display/power service:** display brightness and power-related controls.
- **Keyboard device service:** keyboard backlight and exposed device controls.
- **Shell integration:** Emoji picker, Dictation UI, notifications and settings.
- **Peripheral extension manager:** optional vendor integrations, permissions, lifecycle and isolation.

Do not create one giant keyboard manager or place audio/display/device operations directly in the compositor keyboard event handler. Each module must have one responsibility and focused tests.

## 8. Validation matrix

Automated tests should cover:
- standard HID keyboard and Consumer Control usages;
- volume, mute, media transport and brightness semantics;
- function-row policy and Fn behavior where exposed;
- multiple layouts and non-US key mappings;
- dedicated Emoji and Dictation semantic actions;
- USB, Bluetooth and receiver connection lifecycle;
- multiple simultaneous keyboards and pointing devices;
- device removal while keys or buttons are held;
- unknown and malformed reports;
- missing audio, media, display or shell services;
- extension crash, denial, removal and version mismatch;
- no regression to ordinary typing or application shortcuts.

Physical validation must include representative third-party devices from more than one manufacturer. Logitech MX Keys and MX Master families are useful reference devices, but passing tests on one model does not establish universal peripheral compatibility.

## 9. Implementation order

1. Complete and test the universal input baseline.
2. Establish semantic actions and service contracts.
3. Deliver consistent system behavior and visible feedback.
4. Add device capability discovery and settings.
5. Introduce the isolated extension boundary.
6. Evaluate vendor integrations one at a time, starting only where public documentation or a supported interface exists.
7. Validate end-to-end behavior on physical hardware.

## 10. Reference sources

- Microsoft Learn — Keyboard Input Overview: https://learn.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input
- Microsoft Learn — Top-Level Collections Opened by Windows for System Use: https://learn.microsoft.com/en-us/windows-hardware/drivers/hid/top-level-collections-opened-by-windows-for-system-use
- Apple Support — Use keyboard function keys on Mac: https://support.apple.com/en-la/102439
- Logitech Support — Why advanced F-keys may require software: https://support.logi.com/hc/en-us/articles/5528605895703-Why-do-I-need-to-install-software-to-make-some-advanced-F-keys-work-on-my-keyboard
- Logitech Support — All about Logi Options+: https://support.logi.com/hc/en-001/articles/1500006416522-All-about-Logi-Options
- Logitech — MX Software and app-specific customizations: https://www.logitech.com/en-us/software/mx-software
- Logitech Support — MX Master 4 for Mac: https://support.logi.com/hc/en-150/articles/28321539555351-Getting-Started-MX-Master-4-for-Mac
- Logitech Support — MX Master 4 native haptics with Windows 11: https://support.logi.com/hc/en-in/articles/40581588219671-MX-Master-4-Native-Haptics-with-Windows-11
- Logitech — MX Keys S product specification: https://cdn.cs.1worldsync.com/47/66/47668cd1-9117-4141-9543-c8fc8131f523.pdf
