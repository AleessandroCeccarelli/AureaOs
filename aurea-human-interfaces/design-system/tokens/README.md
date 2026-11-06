<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Tokens

This directory is the source of truth for the approved AureaOS visual token set.

The design system uses a blue-gray foundation with a soft blue primary accent and a restrained warm accent used only for identity highlights. This keeps the desktop familiar, readable, and calm while preserving a clear AureaOS personality.

## Source of truth

- `aurea-human-interfaces/design-system/tokens/index.css`

The `light.css` and `dark.css` files are theme-specific overrides for the shared token set and must stay in sync with the source of truth.

## Token model

Examples of semantic tokens:

- `--aurea-surface-desktop`
- `--aurea-surface-window`
- `--aurea-surface-panel`
- `--aurea-text-primary`
- `--aurea-accent-primary`
- `--aurea-accent-warm`
- `--aurea-dock-bg`
- `--aurea-notification-bg`

## Notes

- The palette is intentionally blue-gray based, not ivory / graphite / gold dominant.
- The warm accent is a secondary identity highlight and should not dominate the shell.
- The default system font remains DA DEFINIRE; Aurea Sans is a future candidate only when approved.
- A final font decision must be made from the repository font database, not from an ad hoc external stack.
