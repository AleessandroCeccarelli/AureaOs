<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Menu Specification

**Status:** Design specification

## Application menu bar
First-party desktop applications use the shared category names and order:
1. File
2. Edit
3. View
4. Navigate
5. Window
6. Help

Italian labels are File, Modifica, Visualizza, Vai, Finestra and Aiuto. Applications may omit a category only when it has no meaningful commands; they must not rename categories or invent synonyms.

## Menu item anatomy
An item may contain an icon, command label, optional shortcut, submenu indicator and state indicator. Keep alignment consistent. Group related commands; avoid unnecessary separators.

## Behavior
Support default, hover, keyboard focus, pressed, selected/checked and disabled states. Arrow keys navigate; Enter activates; Escape closes the current level. Focus returns to the invoking control. Context menus use the same visual language and terminology.

## Themes and accessibility
Use shared menu surface, border, shadow, typography and focus tokens. Expose menu roles, item names, checked state and shortcuts to assistive technology.
