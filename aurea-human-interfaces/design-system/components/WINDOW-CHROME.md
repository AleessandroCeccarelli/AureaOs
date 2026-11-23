<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Window Chrome Specification

**Status:** Design specification
**Primary usability reference:** Familiar Windows desktop conventions

## Shared structure

First-party windows provide:
- Clear application identity and window title.
- Predictable title-bar placement.
- Standard minimize, maximize/restore and close controls.
- Consistent control order, size, hit targets and accessible names.
- A stable relationship between title bar and application menus.

Do not move, rename or redesign standard controls independently in each application. Users must recognize them immediately.

## Visual treatment

Use the shared window surface, border, radius and shadow tokens. The title bar should be visually integrated with the application window, with restrained tonal separation from content. Avoid heavy gradients, thick borders and excessive glass effects. Blur is reserved for floating system surfaces, not ordinary application content.

## Control behavior

- **Minimize:** hides the window while keeping the application represented in the dock's running-app area.
- **Maximize:** fills the available work area without covering reserved system surfaces.
- **Restore:** returns to the previous window geometry.
- **Close:** closes the window and protects unsaved work through the application's normal confirmation/save flow.

Maximize and restore use one control with a clear state. Hover, pressed, focused and disabled states are consistent across applications. Close must remain visually distinguishable without using alarming styling during ordinary use.

## Title and menu relationship

The title identifies the current application and, where relevant, the active document or location. The shared application menu follows the canonical categories and order in `APP-DESIGN-GUIDELINES.md`.

Window controls must not overlap title text or menu items at supported display scales.

## Themes and accessibility

Use shared semantic tokens. Keep title and controls legible in light and dark themes. Provide keyboard alternatives, visible focus and accessible names. Respect reduced motion and reduced transparency.

## Implementation boundary

This document defines visual and interaction expectations. Window movement, focus, decoration ownership, resizing, snapping and presentation belong to compositor and shell, not CSS or application-specific imitation.
