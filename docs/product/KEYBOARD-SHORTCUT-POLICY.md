<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS — Keyboard Shortcut Compatibility Policy

## Purpose

AureaOS adopts familiar, widely used keyboard shortcuts to make the transition from Windows straightforward. The goal is to accompany users in habits they have already learned, not to make them relearn everyday computer interactions.

For common desktop and application actions, AureaOS prefers established Windows 11 conventions over inventing new key combinations. These conventions are familiar, extensively used, and documented by Microsoft. AureaOS keeps its own identity through its design and implementation, not by changing shortcuts without a usability reason.

This is a compatibility and product policy. A shortcut listed here is not automatically evidence that its action is already implemented. Runtime support is tracked explicitly below and must only be marked implemented after the corresponding behavior and tests exist.

## Key terminology

- **Super** is the system modifier corresponding to the Windows logo key on a typical PC keyboard.
- **Application shortcuts** are delivered to the focused application. The compositor must not intercept them unnecessarily.
- **Desktop shortcuts** are handled by AureaOS and must invoke a real, available system action.
- Shortcut labels in the interface should use the user's keyboard layout and familiar key names.

## Everyday application shortcuts

These are common conventions expected to work consistently in compatible applications. They are application-level shortcuts, not global compositor bindings.

| Shortcut | Expected action |
|---|---|
| Ctrl + A | Select all |
| Ctrl + C | Copy |
| Ctrl + X | Cut |
| Ctrl + V | Paste |
| Ctrl + Z | Undo |
| Ctrl + Y | Redo |
| Ctrl + S | Save |
| Ctrl + N | New document or window, where applicable |
| Ctrl + O | Open, where applicable |
| Ctrl + P | Print, where applicable |
| Ctrl + F | Find or search within the current application |
| Ctrl + W | Close the current tab or document; application behavior may vary |
| Ctrl + T | Open a new tab, where the application supports tabs |
| Ctrl + Tab | Move to the next tab, where supported |
| Ctrl + Shift + Tab | Move to the previous tab, where supported |
| F1 | Help, where provided |
| F5 | Refresh or reload, where appropriate |
| Esc | Cancel or dismiss the current operation or dialog |

Applications may add context-specific shortcuts, but should preserve these familiar meanings wherever the action exists. An application must not claim support for an action it does not provide.

## Everyday desktop shortcuts

These are the preferred AureaOS system-wide defaults. The Windows logo key is represented as Super.

| Shortcut | Expected action | Runtime status |
|---|---|---|
| Super | Open the AureaOS launcher | Planned |
| Alt + Tab | Switch between open applications/windows | Planned |
| Shift + Alt + Tab | Switch backwards between applications/windows | Planned |
| Alt + F4 | Close the active window | Planned |
| Super + D | Show the desktop; press again to restore windows | Planned |
| Super + E | Open Fides file manager | Planned |
| Super + I | Open AureaOS Settings | Planned |
| Super + L | Lock the current session | Planned |
| Super + S | Open Quaero search | Planned |
| Super + Tab | Open the desktop/window overview | Planned |
| Super + Up | Maximize the active window | Planned |
| Super + Down | Restore or minimize the active window | Planned |
| Super + Left | Snap the active window to the left | Planned |
| Super + Right | Snap the active window to the right | Planned |
| Super + Ctrl + Left | Switch to the previous virtual desktop | Implemented |
| Super + Ctrl + Right | Switch to the next virtual desktop | Implemented |
| Ctrl + Alt + T | Open the terminal, when installed and enabled | Planned |

"Planned" means the shortcut is part of the compatibility target, but must not be advertised as working until its action is implemented and tested. "Implemented" means the compositor currently resolves the binding; end-to-end behavior must still be covered by the relevant integration tests.

## File manager conventions

Fides should preserve familiar file-management shortcuts where the corresponding operation exists:

| Shortcut | Expected action |
|---|---|
| Ctrl + N | Open a new window |
| Ctrl + T | Open a new tab |
| Ctrl + W | Close the current tab or window |
| Ctrl + Shift + N | Create a new folder |
| F2 | Rename the selected item |
| F3 or Ctrl + F | Search files and folders |
| F5 | Refresh the current view |
| Alt + Left | Go back |
| Alt + Right | Go forward |
| Alt + Up | Go to the parent folder |
| Delete | Move the selected item to Trash |
| Shift + Delete | Permanently delete, with an appropriate confirmation |

These bindings belong to Fides and must not be intercepted as global desktop shortcuts.

## Accessibility and input

AureaOS should provide keyboard access to all essential desktop functions and a discoverable shortcut reference in system settings. Accessibility shortcuts should be introduced alongside the corresponding accessibility services, with clear feedback and safe ways to cancel or disable them.

## Compatibility rules

1. Prefer established Windows 11 shortcuts for common actions.
2. Do not invent alternate combinations merely to make AureaOS different.
3. Do not reserve application-level shortcuts globally unless a desktop-level action genuinely requires it.
4. Do not intercept a shortcut when the corresponding system action is unavailable.
5. Avoid collisions between desktop shortcuts, application shortcuts, keyboard layouts, and accessibility features.
6. Keep shortcuts consistent across AureaOS applications.
7. Any intentional deviation must have a documented usability or accessibility reason.
8. A shortcut becomes "Implemented" only after its action and tests are present.

## Out of scope

AureaOS does not adopt shortcuts for Microsoft-only applications or Windows-specific products and services that are not part of the AureaOS product plan. Examples include Copilot, Xbox Game Bar, Windows Mixed Reality, Microsoft 365-specific commands, and Windows-only diagnostic or management tools.

## Reference sources

The compatibility baseline is informed by Microsoft's published documentation:

- [Microsoft Support — Windows keyboard shortcuts](https://support.microsoft.com/en-us/windows/keyboard-shortcuts-in-windows)
- [Microsoft Support — Windows keyboard tips and tricks](https://support.microsoft.com/en-us/windows/hardware/input-devices/windows-keyboard-tips-and-tricks)

These sources are references for common interaction conventions, not a requirement to reproduce Windows internals or Microsoft-specific features.
