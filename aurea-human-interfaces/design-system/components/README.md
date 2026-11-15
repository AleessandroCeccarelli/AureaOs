<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Component Specifications

This directory is the canonical home for reusable interface component specifications. Each file describes one component family and its visual states, behavior, accessibility requirements and theme behavior.

## Component index

- [Buttons](BUTTONS.md)
- [Text fields](TEXT-FIELDS.md)
- [Menus](MENUS.md)
- [Dialogs](DIALOGS.md)
- [Toggles and selection controls](SELECTION-CONTROLS.md)
- [Window chrome](WINDOW-CHROME.md)

## Rules

- Specifications define behavior and appearance; they do not claim a native implementation exists.
- Use semantic tokens from `../tokens/index.css`; do not introduce private palettes.
- Specify light and dark themes, keyboard focus, disabled states and reduced motion.
- Prefer one responsibility per component family and small, composable controls.
- Shared components must be usable by first-party applications without copying their implementation.
- New shared components must be added to this index.

## Component lifecycle

1. Define the user need and expected behavior.
2. Check whether an existing component already solves it.
3. Specify states, keyboard interaction, accessibility and theme behavior.
4. Add a visual example under `components/examples/` when useful.
5. Add light/dark previews under `previews/components/`.
6. Mark implementation status separately from design approval.
