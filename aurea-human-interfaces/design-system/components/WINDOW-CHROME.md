<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Window Chrome Specification

**Status:** Design specification

## Shared structure
First-party windows use application identity and/or approved icon, predictable title region, standard minimize/maximize-restore/close controls, and consistent spacing, surfaces and borders.

Exact placement follows the approved AureaOS window prototype and native windowing contract. Applications must not relocate or rename standard controls independently.

## Behavior
Controls have consistent hit targets and accessible names. Maximize/restore uses one control and state representation. Closing respects unsaved work. Focused and unfocused windows remain distinguishable without making inactive content unreadable. Resizing and snapping must not overlap title or controls.

## Themes and accessibility
Use shared semantic tokens. Keep title and controls legible in both themes. Provide keyboard alternatives and visible focus.

## Implementation boundary
This is a visual contract. Window movement, focus, decoration ownership, resizing and presentation belong to compositor and shell, not CSS or application-specific imitation.
