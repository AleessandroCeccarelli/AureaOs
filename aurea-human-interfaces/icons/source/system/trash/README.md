<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Desktop Trash Artwork

Standalone desktop waste-bin symbols, separate from the Fides folder/location symbol named `trash` in the v5 folder sprite.

## Visual states

| Theme | Empty | Partially full | Full |
|---|---|---|---|
| Light | `light/trash-empty.svg` | `light/trash-half.svg` | `light/trash-full.svg` |
| Dark | `dark/trash-empty.svg` | `dark/trash-half.svg` | `dark/trash-full.svg` |

The bin is always open and has no lid. Empty shows a visibly empty interior; Half shows crumpled paper inside the opening; Full shows discarded paper protruding above the rim. All variants share the same bin geometry and differ only in contents and theme palette.

## Technical contract

- SVG source masters use a 512 × 512 viewBox and transparent canvas.
- No lid, background tile or opaque canvas rectangle.
- Light/Dark is selected from the active Aurea Appearance theme.
- Empty/Half/Full is selected from the actual Trash contents state, independently of theme.
- Keep editable masters in `source/`; generated exports belong in `generated/`.
- Runtime selection, file discovery, state thresholds and rendering are not implemented by these artwork files.
