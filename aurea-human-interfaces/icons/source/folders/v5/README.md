<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# AureaOS Folder Icon Family

Original AureaOS folder artwork for Fides and the desktop.

- `folder-icons.svg` is the SVG symbol sprite.
- Each icon is addressable as `#light-folder`, `#dark-folder`, `#light-home`, `#dark-home`, and so on.
- 14 semantic icons in each theme: folder, home, user, desktop, documents, downloads, pictures, music, videos, shared, network, system, favorites, trash.
- The generic `folder` is the unmarked gold standard folder.
- Symbols use a shared 512 × 512 viewBox and preserve the approved geometry, colors, and centered emblems.

Use an SVG reference such as `folder-icons.svg#light-downloads` from an SVG `use` element, or extract the symbol into a standalone SVG during packaging. Fides must choose the theme and handle drag/drop state transitions. Runtime animation is intentionally not embedded in the artwork.
