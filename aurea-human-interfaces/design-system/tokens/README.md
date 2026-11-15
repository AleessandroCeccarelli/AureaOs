<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Tokens

This directory is the single source of truth for AureaOS visual tokens.

## Canonical entry point

- `index.css` defines the default light theme and the dark theme override selected with `:root[data-theme='dark']`.
- `light.css` and `dark.css` are compatibility entry points only. They import `index.css` and must not duplicate token values.

Set the system appearance through the root `data-theme` attribute. The runtime appearance service is authoritative.

## Token families

Tokens cover semantic surfaces, text, borders, accent and status colors, focus, dock and panel surfaces, notifications, glass materials, shadows, radii, spacing and typography roles.

## Rules

- Components consume semantic tokens, not private color palettes.
- Light and dark themes preserve the same hierarchy and interaction model.
- Transparency and shadows support hierarchy; they never take priority over contrast.
- A missing token is proposed here before a component introduces a local substitute.
- Keep this directory authoritative; do not create a second theme token source.
