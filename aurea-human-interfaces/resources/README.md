<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Human Interface Resources

The single canonical home for reusable visual and audio resources that are not application/system icon masters or font sources.

## Collections

- `wallpapers/` — desktop wallpaper masters and collection metadata.
- `illustrations/` — onboarding, empty-state and explanatory SVG artwork.
- `textures/` — lightweight theme-aware CSS surface treatments.
- `sounds/` — original sound source library and freedesktop-compatible theme layout.

## Rules

Keep each resource in one canonical collection; do not duplicate it under a parallel `assets/` tree. Preserve editable sources, provenance and license. Keep generated runtime outputs separate from masters. Record dimensions, format and intended use. Avoid baking UI text into illustrations. Visual resources must never be required for readability or accessibility.

## Runtime boundary

This repository stores source material. Packaging, installation, service integration and hardware validation belong to the relevant build/runtime components. A resource is not integrated until that path is implemented and tested.
