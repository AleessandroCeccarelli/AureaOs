<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Desktop Trash Artwork

Standalone, transparent trash-can symbols for the desktop. These are separate from the Fides folder/location symbol named `trash` in the v5 folder sprite.

## Variants

| Theme | Empty | Partially filled | Full |
|---|---|---|---|
| Light | `light/trash-empty.svg` | `light/trash-half.svg` | `light/trash-full.svg` |
| Dark | `dark/trash-empty.svg` | `dark/trash-half.svg` | `dark/trash-full.svg` |

All six assets share the same 512 × 512 geometry, proportions and silhouette. The light and dark variants use theme-adjusted edge and body palettes; the contents state is represented visually without changing the can design.

## Technical contract

- SVG source masters with a transparent canvas; no tile or background rectangle.
- Use the matching theme variant selected by Aurea Appearance.
- Select empty/half/full from the actual Trash contents state; do not infer the state from a theme.
- Keep these masters in `source/`; generated exports belong in `generated/`.
- Runtime selection, file-count thresholds and visual rendering are not implemented by these artwork files.
