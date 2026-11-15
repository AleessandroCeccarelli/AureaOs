<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Icon Library

Owns original AureaOS application, system, launcher and Fides folder artwork. This collection contains editable source masters; generated exports and runtime packaging remain separate.

## Structure

- `source/system/` — launcher and system identity artwork, plus preserved legacy system marks.
- `source/applications/` — first-party application icon masters.
- `source/folders/` — original Fides folder/location artwork, including approved v5.
- `generated/` — reserved for validated exports and runtime packages.
- `FOLDER-ICON-SPEC.md` — folder artwork contract.

Brand presentation artwork belongs in `../brand/`; generic third-party symbols belong in `../symbols/`.

## Current inventory

### Launcher and system artwork

- `source/system/aureaos-launcher-icon.svg` — complete launcher tile.
- `source/system/aureaos-launcher-symbol.svg` — matching transparent launcher symbol.
- `aureaos-icon.svg` and `aureaos-symbol.svg` — preserved legacy AureaOS emblem artwork; do not silently replace these with the launcher variants.
- `aurea-ai-orb.svg`, `file-manager.svg` and `wallet.svg` — retained legacy/system artwork. Their future relationship to the application icon library must be reviewed separately.

### First-party application artwork

The following applications have coordinated complete-tile and transparent-symbol SVG variants under `source/applications/`:

| Application | Complete tile | Transparent symbol |
|---|---|---|
| Quaero | `quaero-icon.svg` | `quaero-symbol.svg` |
| Fides | `fides-icon.svg` | `fides-symbol.svg` |
| Aurea Wallet | `aurea-wallet-icon.svg` | `aurea-wallet-symbol.svg` |
| Aurea Store | `aurea-store-icon.svg` | `aurea-store-symbol.svg` |
| Aurea Time | `aurea-time-icon.svg` | `aurea-time-symbol.svg` |

Aurea Update is not a standalone application: update management belongs in Settings. Its previously created icon files are retained for now but are not part of the approved standalone application inventory. Do not present them as a separate launcher entry.

### Folder artwork

Folder family v5 contains 14 semantic locations, light and dark variants, open/drop states and an SVG sprite. Fides application identity remains separate from folder/location artwork.

## Shared visual contract

Each first-party application icon uses two coordinated variants built from the same symbol geometry, proportions and palette:

- Complete white rounded-square tile for launchers, catalogs and surfaces requiring a full application identity.
- Matching transparent standalone symbol for the dock, panels and compact UI.

The AureaOS launcher mark follows the same two-variant contract. Keep editable masters in `source/`; place generated sizes and runtime packages in `generated/`.

## Production boundary

The presence of an SVG master does not imply that all export sizes, theme states, packaging or runtime loading are complete. Before release, define and validate optical alignment, supported sizes, scaling, contrast, transparent exports and target-shell rendering. Do not modify preserved legacy artwork as part of unrelated icon work.
