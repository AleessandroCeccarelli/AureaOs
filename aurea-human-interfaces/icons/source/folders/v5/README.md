<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# AureaOS Folder Icon Family

Original AureaOS folder artwork for Fides and the desktop.

- `folder-icons.svg` is the SVG symbol sprite.
- Reference symbols with `<use href="folder-icons.svg#light-home"/>` or `#dark-home`.
- 14 semantic icons in each theme: folder, home, user, desktop, documents, downloads, pictures, music, videos, shared, network, system, favorites, trash.
- The generic `folder` is the unmarked gold standard folder.
- Open and drag-target states are included as `light-folder-open`, `light-folder-drop`, `dark-folder-open`, and `dark-folder-drop`.
- Symbols use a shared 512 × 512 viewBox and preserve the approved geometry, colors, and centered emblems.

Fides selects the matching theme symbol and drives open/drop states from actual drag events. Respect reduced-motion preferences. The SVG sprite is source artwork; runtime wiring and transition timing remain in Fides.
