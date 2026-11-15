<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Menu Specification

**Status:** Design specification
**Related rules:** `../../APP-DESIGN-GUIDELINES.md`

## Application menu bar

First-party desktop applications use the same familiar category names and order:

1. File
2. Edit
3. View
4. Navigate
5. Window
6. Help

Italian labels: File, Modifica, Visualizza, Vai, Finestra, Aiuto.

A category may be omitted only when it has no meaningful commands. Do not rename, reorder or replace shared categories with application-specific synonyms. Do not show empty menus.

## Command placement

Keep shared commands in stable locations across applications:
- Undo, redo, cut, copy, paste, select all: Edit.
- Layout, zoom, visibility and presentation: View.
- Location/search navigation: Navigate.
- Window arrangement and switching: Window.
- Documentation, support and version information: Help.

Application-specific commands belong in the closest relevant category. Contextual actions may also appear in right-click menus. Do not duplicate the same command under conflicting names.

## Menu item anatomy

An item may contain an icon, command label, optional shortcut, submenu indicator and checked/state indicator. Align labels and shortcuts consistently. Group related commands and avoid unnecessary separators.

## Surface and appearance

Floating menus use a shared elevated surface with a subtle border, rounded corners and soft shadow. A restrained translucent material may be used for menus detached from dense content, but menu text and icons remain crisp and high-contrast. Use the shared glass tokens only where the backdrop and compositor support them; otherwise use an opaque elevated surface. Avoid bright gradients inside menu rows.

## Behavior

Support pointer hover, keyboard focus, pressed, checked and disabled states. Arrow keys navigate; Enter activates; Escape closes the current menu level. Focus returns to the invoking control. Menus stay within screen bounds and open in predictable directions.

Context menus appear near the pointer when possible and expose actions relevant to the clicked object or surface.

## Themes and accessibility

Use shared menu surface, border, shadow, typography and focus tokens. Expose menu roles, item names, checked state and shortcuts to assistive technology. Maintain readable contrast in both themes and support reduced transparency.
