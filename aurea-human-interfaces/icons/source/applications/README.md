<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# First-Party Application Icon Sources

Original AureaOS application icon masters live here. Keep one focused source file per variant and application; document viewBox, export sizes, theme behavior and provenance.

## Variant contract

Each approved first-party application uses two coordinated SVG variants built from the same visual symbol:

- `<application>-icon.svg` — complete icon tile for launchers, catalogs and places that need a full application identity.
- `<application>-symbol.svg` — transparent standalone symbol for the dock, panels, compact controls and other UI surfaces.

The symbol geometry, proportions and colors must remain consistent between the two variants. Do not substitute generic Fluent symbols for original application identities; Fluent UI System Icons remain the shared system-symbol library.

## Approved application inventory

### Quaero

- `quaero-icon.svg` — complete white rounded-square tile with layered translucent blue surfaces and a glossy cyan/blue magnifying glass.
- `quaero-symbol.svg` — matching magnifying-glass artwork on a transparent canvas.
- Both masters use a 1024 × 1024 viewBox and are self-contained SVGs without external image dependencies.

### Fides

- `fides-icon.svg` — complete white rounded-square tile with a glossy dimensional blue/cyan folder and layered document sheets.
- `fides-symbol.svg` — matching folder-and-documents artwork on a transparent canvas.
- Both masters use a 1024 × 1024 viewBox and are self-contained SVGs without external image dependencies.
- Keep this application identity separate from the Fides folder/location icon family in `source/folders/`.

### Aurea Wallet

- `aurea-wallet-icon.svg` — complete white rounded-square tile with the canonical deep-blue shield, layered silver/cyan bevels and dark luminous keyhole.
- `aurea-wallet-symbol.svg` — matching shield-and-keyhole artwork on a transparent canvas.
- Both masters use a 1024 × 1024 viewBox and are self-contained SVGs without external image dependencies.

### Aurea Store

- `aurea-store-icon.svg` — complete white rounded-square tile with layered blue/cyan surfaces and a glossy dimensional shopping bag.
- `aurea-store-symbol.svg` — matching shopping-bag artwork on a transparent canvas.
- Both masters use a 1024 × 1024 viewBox and are self-contained SVGs without external image dependencies.

### Aurea Time

- `aurea-time-icon.svg` — complete white rounded-square tile with two sculpted metallic blue/cyan arrows chasing each other around layered version plates.
- `aurea-time-symbol.svg` — matching circular-arrow and version-layer artwork on a transparent canvas.
- Both masters use a 1024 × 1024 viewBox and are self-contained SVGs without external image dependencies.

## Retained but not an approved standalone application

Aurea Update is a system update function exposed through Settings, not a separate application in the current product direction. The existing `aurea-update-icon.svg` and `aurea-update-symbol.svg` files are retained pending a separate decision about whether either asset has a valid in-Settings use. Do not list Aurea Update as a launcher application or create a separate app entry from these files.

## Shared artwork direction

The application family uses a white rounded-square tile, dimensional blue/cyan artwork, controlled glass-like surfaces and consistent lighting. The transparent symbol must preserve the same underlying geometry and visual identity without the tile.

## Source and runtime rules

- Keep SVG masters self-contained and free of external image dependencies.
- Preserve the two-variant relationship when revising an application mark.
- Do not treat a raster preview as an editable SVG master.
- Generated sizes and runtime packages belong in `../generated/`, not beside the source masters.
- A source SVG is not evidence of installed or visually validated runtime integration.

All original artwork is GPL-3.0-or-later unless a file-specific notice says otherwise.
