<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Theme Compatibility Layer

The canonical theme/token source is `../design-system/tokens/index.css`. This directory exists only for compatibility with existing web prototypes and consumers of legacy theme paths; it must not own a second palette.

- `light/theme.css` — imports canonical tokens and exposes legacy aliases for light mode.
- `dark/theme.css` — imports canonical tokens and exposes the same aliases for dark mode.
- `tokens/index.css` — compatibility entry point importing both wrappers.

Add or change actual colors only in `design-system/tokens/index.css`. Compatibility aliases must reference canonical variables with `var()`. New components should use canonical names directly. Remove an alias only after consumers migrate. These CSS files do not imply the native shell theme service is implemented.
