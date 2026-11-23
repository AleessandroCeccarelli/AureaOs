<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Desktop Shell Pattern

**Status:** Design specification
**Full specification:** [Desktop Experience](../../DESKTOP-EXPERIENCE-SPEC.md)

## Purpose

The shell provides familiar access to applications, open windows, workspaces and system status. It should feel immediately understandable to a Windows desktop user while retaining a distinctive AureaOS finish.

## Composition

- Floating bottom dock with familiar taskbar behavior.
- Aurea A launcher entry at the left.
- Pinned and running applications with clear active-window indication.
- Clock, connectivity, audio, power and notification access at the right.
- Floating launcher, Quick Settings and notification surfaces.
- Four workspaces available as an additional organization tool.

## Shared visual language

The shell uses canonical tokens for color, spacing, radii, borders, shadows and typography. Keep the desktop canvas calm and the dock visually light. The dock, launcher and Quick Settings may use translucent glass with backdrop blur, soft edge highlights and layered shadows. Notification cards use the dedicated notification glass material specified in [Feedback and Notifications](FEEDBACK-AND-NOTIFICATIONS.md).

Glass is reserved for floating shell surfaces. Application content remains readable and stable. If blur is unavailable, use an opaque elevated surface. Under reduced transparency, remove blur and translucent highlights without losing hierarchy.

## Quick Settings — Night Light

The right side of the floating dock opens Quick Settings. Include a Night Light control with:

- A clear on/off state and accessible label.
- A direct toggle that does not require opening Settings.
- A compact indication of the selected warmth level.
- A route to the complete schedule and intensity controls in Settings → Appearance.
- A visible unavailable/unsupported state if the compositor cannot apply the output-wide adjustment.

Quick Settings and Settings must display the same system preference state. They must not maintain separate copies of the Night Light setting.

## Interaction

Super opens the launcher. Alt+Tab switches open windows. The dock distinguishes pinned applications, running applications and minimized windows. The launcher provides search, pinned apps and a discoverable full application list.

The dock may float, remain visible or auto-hide. These presentation modes must not change the familiar meaning of pinned and running applications.

## Motion and accessibility

Use brief, purposeful transitions for opening, closing, hover and workspace changes. Respect reduced motion. Preserve keyboard access, visible focus, readable contrast and display scaling in both themes.

## Status

This pattern documents intended behavior. It does not assert that shell surfaces, shortcuts or service integrations are already implemented.
