<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# AureaOS Desktop Trash Artwork

Standalone desktop Trash artwork, separate from the Fides v5 folder/location symbol.

## Visual direction
The desktop Trash depicts a real open cylindrical wire-mesh wastebasket:
- Slightly tapered body with a rounded, rolled metallic rim and no lid.
- Crossed metal mesh with deliberate transparency, so the interior and contents remain visible.
- Subtle metallic highlights, side shading and a reinforced lower ring.
- Identical silhouette and perspective in all states and themes.
- Transparent canvas, without a background tile.

## States
| Theme | Empty | Partially full | Full |
|---|---|---|---|
| Light | light/trash-empty.svg | light/trash-half.svg | light/trash-full.svg |
| Dark | dark/trash-empty.svg | dark/trash-half.svg | dark/trash-full.svg |

Empty shows the unobstructed interior. Half shows crumpled paper inside. Full shows paper rising above the rim. Contents are layered behind the front mesh. Light/Dark changes only the metallic palette; Empty/Half/Full changes only the contents.

## Technical contract
- 512 × 512 viewBox; transparent canvas.
- Light/Dark follows Aurea Appearance.
- Empty/Half/Full follows actual Trash contents, independently of theme.
- Editable masters stay in source/; validated exports belong in generated/.
- Runtime selection, file discovery, thresholds and rendering are not implemented here.
- Validate at desktop icon sizes and over light/dark wallpapers before final approval.
