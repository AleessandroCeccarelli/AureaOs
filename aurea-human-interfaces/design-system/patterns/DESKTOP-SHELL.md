<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Desktop Shell Pattern

**Status:** Design specification  
**Full specification:** [Desktop Experience](../../DESKTOP-EXPERIENCE-SPEC.md)

## Purpose
The shell provides persistent access to applications, workspaces and system status without competing with active work.

## Composition
- Full-width intelligent dock at the bottom.
- Aurea A launcher entry and workspace navigation on the left.
- Running and pinned applications in the center.
- Clock, connectivity, audio, power and system-panel entries on the right.
- Floating launcher, notifications and quick settings as system-owned surfaces.

## Interaction
The launcher opens from the dock's A mark or Super key. The dock adapts to available workspace and user visibility preference. Workspaces remain discoverable. System panels display real service state.

## Consistency
The dock, launcher, panels and windows use shared tokens and motion rules. No application may draw a competing system dock or launcher.

## Status
This pattern documents the intended experience; it does not assert that every shell surface or service integration is implemented.
