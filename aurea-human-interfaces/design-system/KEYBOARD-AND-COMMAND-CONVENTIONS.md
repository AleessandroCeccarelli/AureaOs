<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Keyboard and Command Conventions

**Status:** Design specification  
**Primary reference:** Windows desktop keyboard conventions  
**Scope:** Desktop shell and first-party applications

## 1. Purpose

AureaOS preserves familiar PC keyboard behavior. Users should be able to transfer their existing Windows desktop habits without relearning common operations.

This document is the shared source of truth for shortcut ownership and command behavior. Applications may add local shortcuts, but must not redefine a shared shortcut or intercept a shell-owned shortcut.

## 2. Modifier terminology

- **Super:** the system key commonly marked with the Windows logo. In user-facing help, call it the **Super / Windows key**.
- **Ctrl:** Control.
- **Alt:** Alt / Option where relevant to the physical keyboard.
- **Shift:** Shift.

Show the actual key glyphs available on the user's keyboard layout. Localize shortcut descriptions, not the underlying command identity.

## 3. Desktop and system shortcuts

| Shortcut | Expected action | Owner |
|---|---|---|
| Super | Open/close application launcher | Shell |
| Super + E | Open Fides | Shell |
| Super + L | Lock session | Shell |
| Super + D | Show desktop / minimize or restore windows | Shell |
| Super + Tab | Open desktop overview | Shell |
| Alt + Tab | Switch between open windows | Shell |
| Alt + F4 | Close active window | Shell/window manager |
| Super + Left / Right | Snap active window to left / right | Shell/compositor |
| Super + Up | Maximize active window | Shell/compositor |
| Super + Down | Restore or minimize according to window state | Shell/compositor |
| Ctrl + Alt + Delete | Open secure session/power options | Shell; exact action must be explicit |
| Print Screen | Open screenshot capture workflow | Shell/service |
| Screen recording shortcut | Not assigned yet; reserve a configurable shell-owned action after conflict review | Shell/service (planned) |

If hardware, compositor or service support is unavailable, do not display a shortcut as functional. The exact Super+Tab overview and Super+arrow snap behavior must be implemented and tested before release.

## 4. Common application shortcuts

| Shortcut | Expected action |
|---|---|
| Ctrl + A | Select all |
| Ctrl + C | Copy |
| Ctrl + X | Cut |
| Ctrl + V | Paste |
| Ctrl + Z | Undo |
| Ctrl + Y | Redo |
| Ctrl + S | Save |
| Ctrl + Shift + S | Save as |
| Ctrl + P | Print, where supported |
| Ctrl + F | Find/search within the current application |
| Ctrl + N | New document/window/item, according to application context |
| Ctrl + O | Open |
| Ctrl + W | Close current document/tab/view |
| Ctrl + T | New tab, where tabs are supported |
| Ctrl + Shift + T | Reopen recently closed tab, where supported |
| F1 | Help, where available |
| F2 | Rename selected item in file-management contexts |
| F5 | Refresh/reload current view, where appropriate |
| Delete | Move selected item to trash or remove from current context, with clear feedback |

Applications must not assign conflicting meanings to shared shortcuts without a documented, user-visible reason.

## 5. Fides file-manager shortcuts

Fides follows familiar file-manager conventions:

| Shortcut | Expected action |
|---|---|
| Ctrl + L | Focus location/path entry |
| Ctrl + Shift + N | Create new folder |
| Alt + Left | Back |
| Alt + Right | Forward |
| Alt + Up | Parent folder |
| F2 | Rename selected item |
| Delete | Move selected item to Trash |
| Shift + Delete | Permanently delete, with explicit confirmation |
| Ctrl + C / X / V | Copy / cut / paste |
| Ctrl + A | Select all |
| Ctrl + F | Search within Fides |

A shortcut must not silently perform an irreversible operation. Permanent deletion requires clear confirmation.

## 6. Shortcut ownership and conflicts

- The shell owns global shortcuts and reserves them before applications receive input.
- Applications own shortcuts scoped to their focused window or active control.
- A global shortcut must not be shadowed by an application.
- When a shortcut is unavailable or conflicts with a hardware/layout constraint, provide an equivalent discoverable menu command.
- Shortcut hints in menus must match actual behavior.
- Do not advertise a shortcut until the runtime action exists.
- User remapping may be added later, but defaults remain familiar and stable.

## 7. Menu and command consistency

Common commands must appear in consistent menu categories and use consistent wording. The menu bar follows File, Edit, View, Navigate, Window, Help; localized labels must preserve the same order and meaning.

A command available from a context menu should use the same label and shortcut as its equivalent in the application menu.

## 8. Validation checklist

- [ ] Super opens and closes the launcher.
- [ ] Alt + Tab switches windows.
- [ ] Alt + F4 closes the active window.
- [ ] Ctrl + C / X / V work consistently.
- [ ] Fides common shortcuts match the table.
- [ ] Global shortcuts cannot be intercepted by applications.
- [ ] Menu hints match actual key bindings.
- [ ] Shortcuts are tested with supported keyboard layouts.
- [ ] No shortcut is documented as implemented before runtime support exists.
- [ ] Screenshot source selection and cancellation are keyboard accessible.
- [ ] Recording status and stop action remain reachable from the shell.
- [ ] No recording shortcut is advertised until its key binding and runtime action are implemented.

**Principle:** familiar shortcuts are part of the user interface. Preserve them consistently, document ownership and never promise behavior that the system does not yet implement.
