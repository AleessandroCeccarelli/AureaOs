<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Desktop Shell Pattern

**Status:** Design specification  
**Full specification:** [Desktop Experience](../../DESKTOP-EXPERIENCE-SPEC.md)

## Purpose

The shell provides familiar access to applications, open windows, workspaces and system status. It should feel immediately understandable to a Windows desktop user.

## Composition

- Floating bottom dock with familiar taskbar behavior.
- Aurea A launcher entry at the left.
- Pinned and running applications with clear active-window indication.
- Clock, connectivity, audio, power and notification access at the right.
- Floating launcher, quick settings and notification surfaces.
- Four workspaces available as an additional organization tool.

## Interaction

Super opens the launcher. Alt+Tab switches open windows. The dock distinguishes pinned applications, running applications and minimized windows. The launcher provides search, pinned apps and a discoverable full application list.

The dock may float, remain visible or auto-hide. These presentation modes must not change the familiar meaning of pinned and running applications.

## Visual consistency

Use shared tokens, window controls, menu rules and motion behavior. The dock may take visual cues from KDE Plasma; shell consistency and notification refinement may take cues from macOS. These references must not make ordinary desktop tasks unfamiliar.

## Status

This pattern documents intended behavior. It does not assert that shell surfaces, shortcuts or service integrations are already implemented.
