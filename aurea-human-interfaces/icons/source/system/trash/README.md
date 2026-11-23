<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# AureaOS Desktop Trash Artwork

Standalone desktop Trash artwork, separate from the Fides v5 folder/location symbol.

## Approved visual direction

The desktop Trash follows the supplied visual reference: a simple, open, softly frosted translucent white wastebasket.

- Gently tapered cylindrical body with a softly rounded base and understated rolled rim.
- No lid, no wire mesh, no perforations and no opaque background tile.
- Satin/opal white material. The interior and discarded objects are softly diffused but perceptible through the translucent body, similar to the visual treatment in Windows and macOS.
- A clear open top; when full, paper and small discarded items visibly rise above the rim.
- The same silhouette, proportions, lighting and perspective in every state and theme.
- Transparent SVG canvas; the desktop wallpaper is never part of the icon artwork.

## States

| Theme | Empty | Partially full | Full |
|---|---|---|---|
| Light | light/trash-empty.svg | light/trash-half.svg | light/trash-full.svg |
| Dark | dark/trash-empty.svg | dark/trash-half.svg | dark/trash-full.svg |

Empty has a clean, visible interior. Half contains a modest amount of softly visible paper. Full has several recognisable pieces of paper protruding above the rim. Theme changes only the subtle material contrast; state changes only the contents.

## Technical contract

- 512 × 512 viewBox; transparent canvas.
- Light/Dark follows Aurea Appearance.
- Empty/Half/Full follows actual Trash contents, independently of theme.
- Keep editable masters in source/; validated exports belong in generated/.
- Runtime selection, file discovery, thresholds and rendering are not implemented here.
- Inspect the rendered assets over both light and dark wallpapers and at actual desktop icon sizes before final approval.
