<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# First-Party Application Icon Sources

Original AureaOS application icon masters live here. Keep one focused source file per variant and application; document viewBox, export sizes, theme behavior and provenance.

## Variant contract

Each first-party application uses two coordinated SVG variants built from the same visual symbol:

- `<application>-icon.svg` — complete icon tile for launchers, catalogs and places that need a full application identity.
- `<application>-symbol.svg` — transparent standalone symbol for the dock, panels, compact controls and other UI surfaces.

The symbol geometry, proportions and colors must remain consistent between the two variants. Do not substitute generic Fluent symbols for original application identities; Fluent UI System Icons remain the shared system-symbol library.

## Quaero

- `quaero-icon.svg` — complete white rounded-square tile with layered translucent blue surfaces and a glossy cyan/blue magnifying glass.
- `quaero-symbol.svg` — the matching magnifying-glass artwork on a transparent canvas.
- Both masters use a 1024 × 1024 viewBox and are authored as self-contained SVGs without external image dependencies.
- Artwork direction: Fluent-inspired dimensional finish, Windows 11-style material softness, with an AureaOS blue/cyan identity.
- SPDX: GPL-3.0-or-later.
