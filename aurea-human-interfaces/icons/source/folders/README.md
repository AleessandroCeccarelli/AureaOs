<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS folder icon family

Original, standalone SVG source artwork for Fides locations and standard user folders.

## Assets

| File | Intended location |
|---|---|
| `system.svg` | AureaOS system location |
| `desktop.svg` | Desktop |
| `home.svg` | User home |
| `documents.svg` | Documents |
| `downloads.svg` | Downloads |
| `pictures.svg` | Pictures |
| `music.svg` | Music |
| `videos.svg` | Videos |
| `network.svg` | Network and shared locations |
| `folder.svg` | Generic folder fallback |

## Artwork rules

- Keep each icon as an independent, editable SVG with a transparent canvas.
- Use the AureaOS metallic blue, silver and cyan folder silhouette.
- Keep category glyphs centered, simple and legible at reduced sizes.
- These are folder/location icons, not application launchers. The Fides application icon remains in `source/system/file-manager.svg`.
- The network icon identifies a location only; it does not imply that a share is connected or mounted.

## Status

Initial source family. Review optical balance and small-size rendering before runtime integration.
