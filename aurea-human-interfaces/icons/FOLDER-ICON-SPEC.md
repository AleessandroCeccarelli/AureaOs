<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Fides folder icon specification

## Purpose

Define the requirements for the folder and location artwork used by Fides and the AureaOS desktop. This document is an acceptance contract, not a claim that artwork or runtime integration is complete.

## Visual direction

- Prefer the familiar, immediately readable folder language used by modern Windows desktops.
- Use one coherent icon family: consistent folder silhouette, perspective, lighting, edge treatment, corner geometry and visual weight.
- Category marks must be recognizable without turning each icon into a different illustration style.
- AureaOS may adapt the palette and distinctive details to its own identity. Do not copy a third-party icon drawing or brand asset.
- Keep the application icon for Fides separate from folder/location artwork.

## Required initial locations

| Semantic identifier | User-facing label (English) | Meaning |
|---|---|---|
| `folder` | Folder | Generic folder fallback |
| `desktop` | Desktop | User desktop directory |
| `home` | Home | User's home directory |
| `documents` | Documents | Personal documents |
| `downloads` | Downloads | Downloaded files |
| `pictures` | Pictures | Images and photographs |
| `music` | Music | Audio and music files |
| `videos` | Videos | Video files |
| `network` | Network | Network locations and shared resources |
| `system` | AureaOS | Clearly identified system-owned location, if exposed in Fides |

Optional later categories (Archives, Favorites, Trash, removable drives and cloud locations) require separate product decisions. Do not imply a network share is connected or available merely by displaying its icon.

## Source and export requirements

- Prefer editable SVG source with a square `viewBox="0 0 256 256"` and transparent background.
- Artwork must remain crisp when rendered at 16, 24, 32, 48, 64, 96, 128 and 256 logical pixels.
- Keep the primary silhouette and category mark optically centered, with consistent safe margins.
- Avoid embedded raster images, external resources, fonts, scripts, filters with excessive rendering cost, and hard-coded background rectangles.
- If a source family provides raster-only artwork, retain the original source and document the approved export sizes; do not pretend it is vector artwork.
- Small-size variants may be introduced only when testing demonstrates that simplification is necessary.

## Licensing and provenance gate

Before an asset is added:
1. Record the exact source project and asset/family name.
2. Record the license applying to that exact asset and its version.
3. Confirm modification, commercial use, redistribution and inclusion in a GPL-3.0-or-later operating-system repository are permitted.
4. Preserve required copyright, attribution and license notices.
5. Do not commit assets whose terms prohibit redistribution of the artwork, including modified artwork.
6. Keep third-party assets separate from original AureaOS artwork and do not relicense them as GPL if their license does not permit it.

A free download or an editable SVG does not by itself establish redistribution rights.

## Review checklist

- [ ] A complete, visually coherent family has been selected.
- [ ] Provenance and redistribution rights are documented for every asset.
- [ ] Icons are visually consistent at 256 px.
- [ ] Icons remain distinguishable at 48 px and 32 px.
- [ ] 24 px and 16 px rendering has been reviewed; simplified variants are supplied if needed.
- [ ] Light and dark backgrounds have been checked.
- [ ] Fides application icon remains distinct from folder icons.
- [ ] Approved assets are mapped to semantic identifiers before runtime integration.

## Status

Artwork selection is pending. No folder artwork is approved by this specification, and no runtime integration is implied.
