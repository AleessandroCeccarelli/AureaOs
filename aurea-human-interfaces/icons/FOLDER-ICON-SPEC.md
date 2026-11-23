<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# Fides folder icon specification

## Purpose
Define the folder and location artwork used by Fides and the AureaOS desktop.

## Visual direction
- One coherent family with consistent folder silhouette, perspective, lighting, edge treatment, corner geometry and visual weight.
- The approved AureaOS family uses the standard gold folder as its geometry reference; category icons retain the same proportions and differ by palette and emblem.
- Category marks are optically centered on the folder face and use a layered dimensional treatment.
- Separate light and dark theme palettes are provided.
- Keep the Fides application icon separate from folder/location artwork.

## Required locations
| Identifier | Meaning |
|---|---|
| `folder` | Generic folder fallback |
| `home` | User home directory |
| `user` | User profile/home identity |
| `desktop` | Desktop directory |
| `documents` | Personal documents |
| `downloads` | Downloaded files |
| `pictures` | Images and photographs |
| `music` | Audio and music files |
| `videos` | Video files |
| `network` | Network locations |
| `shared` | Shared folders |
| `system` | AureaOS system location |
| `favorites` | Favorites |
| `trash` | Trash |

## Technical and runtime requirements
- SVG source uses a square 512 × 512 viewBox and transparent background.
- Artwork should be validated at 16, 24, 32, 48, 64, 96, 128 and 256 logical pixels.
- Fides selects the matching light/dark symbol from the active system theme.
- Open/drop animation is driven by actual drag-and-drop events in Fides; honor reduced-motion preferences.
- The symbol sprite is stored in `source/folders/v5/folder-icons.svg`.

## Licensing and provenance
Original AureaOS artwork. Copyright (c) 2026 Alessandro Ceccarelli and AureaOS contributors. Licensed under GPL-3.0-or-later; see `source/folders/v5/LICENSE.md`. No Folder11 artwork or third-party brand logos are included.

## Status
Artwork family approved for repository inclusion. Runtime integration and visual testing in Fides remain separate tasks.
