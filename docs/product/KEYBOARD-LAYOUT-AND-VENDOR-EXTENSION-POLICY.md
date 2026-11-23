<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS — Keyboard, Pointer and Vendor Extension Policy

## Purpose

AureaOS must provide universal keyboard and pointing-device behavior independent of manufacturer software. Layout selection, pointer movement, standard buttons, scrolling and ordinary input behavior belong to AureaOS. Manufacturer integrations are optional extensions for additional hardware-specific capabilities.

This document defines the intended architecture. It does not claim that dynamic layout management or vendor extensions are already implemented.

## 1. Separate hardware, layout, language and text

AureaOS must keep these concepts distinct:

- **Physical device:** the keyboard and its transport (built-in, USB, Bluetooth or supported receiver).
- **Physical key position/code:** which physical key generated an event.
- **Logical key symbol:** the meaning assigned by the active XKB keymap.
- **Input layout/source:** the selected keyboard mapping, such as Italian, US English or German.
- **Input language and composition:** language-related input methods, compose sequences, candidate selection and text composition.
- **Produced text:** the final text delivered to the focused application.

A keyboard's USB/HID identity does not reliably reveal the language the user wants to type. AureaOS must never silently assume that a vendor or product ID determines the correct layout.

## 2. Universal software-managed layout

The operating system owns layout configuration through a dedicated Keyboard Layout Manager using XKB.

Required behavior:

- provide selectable layouts and variants, including non-US layouts;
- allow multiple installed input sources and quick switching;
- apply the selected keymap consistently to all ordinary keyboards unless the user configures a per-device override;
- preserve physical key identity separately from the resulting symbol and text;
- support modifiers, compose behavior, dead keys and layout-specific punctuation;
- expose a clear active-layout indicator and settings UI;
- persist user preferences and restore them across sessions;
- handle layout changes without leaving modifiers or keys logically held.

### Initial layout selection

The first-run experience should propose a sensible default using the user's selected system language/region and any trustworthy layout metadata actually exposed by the hardware or firmware.

If evidence is incomplete or conflicting, ask the user to confirm the physical layout. Do not claim automatic language detection when the device provides no reliable signal.

AureaOS may offer an optional guided typing check to help the user identify a physical layout. This must be local, transparent and must not transmit typed content.

### Multiple keyboards

The default policy is one active input source for the session, shared across keyboards. A future per-device layout override may be offered for users who connect keyboards with different physical layouts. Such overrides must be explicit and visible; device identity alone must not trigger an unexpected layout switch.

## 3. Universal keyboard and pointer behavior

The core OS must handle standard keyboard, consumer-control and pointing-device events without a vendor extension wherever the device exposes standard HID/Linux input events.

Keyboard behavior includes ordinary typing, navigation, modifiers, function keys and standard media/audio controls. AureaOS semantic actions route system controls to their owning services; the compositor must not implement volume, brightness, media or keyboard-backlight operations directly.

Mouse and pointer behavior includes relative motion, primary/secondary/middle buttons, supported back/forward buttons, vertical and horizontal scrolling, and high-resolution scroll data when exposed by the device. Standard pointer acceleration and speed preferences belong to AureaOS and must work without vendor software. Touchpad-specific capabilities such as tapping, natural scrolling and gestures must be advertised only when the hardware and input backend expose them.

Applications receive ordinary focused-surface input through Wayland. They must not receive unrestricted global raw keyboard or pointer streams through the system API. Global shortcuts and privileged device configuration remain system-owned and permission-controlled.

Unknown keys and buttons must remain safe and configurable where possible. They must not be converted into arbitrary text or interpreted as commands.

## 4. Manufacturer extension model

AureaOS should publish a stable, versioned Peripheral Extension API. Manufacturers may provide optional packages for supported devices.

Potential extension features include:

- remapping extra keyboard keys, mouse buttons and supported touchpad gestures;
- per-application profiles;
- programmable gestures and action rings;
- device lighting and advanced backlight effects;
- battery and receiver status;
- supported haptic controls;
- device configuration and firmware workflows;
- coordinated behavior across multiple devices.

The extension API must be capability-based. An extension declares the device families and operations it supports; AureaOS verifies the declaration against the connected device and the available system interfaces. A marketing name or USB vendor/product ID alone is not proof that a feature is supported.

## 5. Security and process boundaries

- Vendor extensions must never execute inside the compositor process.
- Prefer an out-of-process service with a narrow, versioned IPC contract.
- Grant only the specific capabilities approved by the user and required by the extension.
- Do not expose raw keystrokes, typed text, passwords, clipboard contents or unrestricted application data to an extension.
- Require explicit approval for command execution, application-context access, data transfer, system-setting changes and firmware updates.
- Validate all messages and device reports; enforce resource limits and rate limits.
- Firmware operations must use a supported, recoverable update path and require explicit user confirmation.
- Extension failure, removal, denial or incompatibility must not affect standard typing or pointer operation.
- The system must provide a way to inspect, disable and uninstall each extension.

## 6. API and distribution requirements

The future extension contract must define:

- API version negotiation and compatibility policy;
- device matching and capability declaration;
- permission manifest and user-consent flow;
- semantic actions and settings registration;
- lifecycle, hotplug and disconnect notifications;
- logging and diagnostics that exclude typed content;
- package provenance, signature verification and update policy;
- graceful behavior when the extension is absent or outdated.

The AureaOS Store may distribute approved manufacturer packages. A manufacturer may also publish packages through another trusted channel, provided they meet the same signing, permission and compatibility requirements.

AureaOS must not claim compatibility with proprietary protocols unless the manufacturer documents or licenses a supported interface and the integration has been independently validated.

## 7. Responsibility boundaries

- **Input backend:** device events and lifecycle.
- **Device discovery manager:** identity and exposed capabilities.
- **Keyboard Layout Manager:** XKB keymaps, layouts, variants and switching.
- **Input method integration:** composition and language-specific text entry.
- **Input normalization:** stable semantic events.
- **Shortcut resolver:** global desktop shortcuts.
- **System services:** audio, media, display and keyboard-device operations.
- **Peripheral Extension Manager:** extension discovery, permissions, lifecycle and IPC.
- **Shell:** settings, layout indicator, emoji picker and user feedback.

Each responsibility must live in a small, focused module. Do not create a monolithic keyboard manager or move device control into the compositor event handler.

## 8. Implementation sequence

1. Audit the existing Smithay/XKB keyboard path and identify the current keymap configuration boundary.
2. Introduce a small software-managed layout model and explicit XKB configuration flow.
3. Add layout selection, persistence and switching UI through the shell only when its settings contract is ready.
4. Validate Italian, US English, German and other non-US layouts, including dead keys and compose behavior.
5. Complete universal standard-key and Consumer Control routing to real system services.
6. Specify and test the versioned extension API and permission model.
7. Implement the extension manager out of process.
8. Integrate manufacturer packages individually, only against documented interfaces and real hardware.

## 9. Current implementation status

The compositor currently initializes its Smithay keyboard with `XkbConfig::default()`. Its keyboard event path resolves global shortcuts and forwards other events to the focused Wayland client. The current input backend dispatches keyboard events from libinput.

This is a baseline, not a complete layout-management system. The repository must not describe automatic layout selection, vendor extensions or advanced device configuration as implemented until their modules, settings and end-to-end tests exist.

## 10. Acceptance criteria

- A standard keyboard works without manufacturer software.
- The user can select and switch layouts in software.
- Italian, US English and German layouts produce the expected symbols, punctuation, dead keys and compose behavior.
- Layout selection does not depend on the keyboard brand.
- Multiple keyboards work without stuck modifiers during hotplug or removal.
- A missing or crashed vendor extension does not affect normal input.
- An extension cannot read typed text or access unrelated desktop data.
- An extension can only invoke explicitly granted, declared capabilities.
- Unknown or unsupported device features fail safely.
