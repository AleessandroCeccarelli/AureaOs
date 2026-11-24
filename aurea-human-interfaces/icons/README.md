<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# AureaOS Icon Library

Owns original AureaOS application, system, launcher and Fides folder artwork. This collection contains editable source masters; generated exports and runtime packaging remain separate.

## Structure
- source/system/ — launcher, system identity and desktop system artwork.
- source/applications/ — first-party application icon masters.
- source/folders/ — original Fides folder/location artwork, including approved v5.
- generated/ — reserved for validated exports and runtime packages.
- FOLDER-ICON-SPEC.md — folder artwork contract.

## Current inventory

### Launcher and system artwork
- source/system/aureaos-launcher-icon.svg — complete launcher tile.
- source/system/aureaos-launcher-symbol.svg — matching transparent launcher symbol.
- source/system/trash/ — six transparent desktop Trash states: empty, half-full and full for Light and Dark themes. The artwork depicts an open, softly frosted translucent white wastebasket; its interior and contents remain gently visible through the body.
- aureaos-icon.svg and aureaos-symbol.svg — preserved legacy AureaOS emblem artwork; do not silently replace these with launcher variants.
- aurea-ai-orb.svg, file-manager.svg and wallet.svg — retained legacy/system artwork; their future relationship to the application icon library must be reviewed separately.

### First-party application artwork
| Application | Complete tile | Transparent symbol |
|---|---|---|
| Quaero | quaero-icon.svg | quaero-symbol.svg |
| Fides | fides-icon.svg | fides-symbol.svg |
| Aurea Wallet | aurea-wallet-icon.svg | aurea-wallet-symbol.svg |
| Aurea Store | aurea-store-icon.svg | aurea-store-symbol.svg |
| Aurea Time | aurea-time-icon.svg | aurea-time-symbol.svg |

Aurea Update is not a standalone application: update management belongs in Settings. Its previously created icon files are retained but are not part of the approved standalone application inventory.

### Folder artwork
Folder family v5 contains 14 semantic locations, light and dark variants, open/drop states and an SVG sprite. Fides application identity remains separate from folder/location artwork.

## Shared visual contract
First-party application icons use coordinated complete-tile and transparent-symbol variants. Keep editable masters in source/; place generated sizes and runtime packages in generated/.

## Production boundary
SVG masters do not imply validated exports, packaging or runtime loading. Validate optical alignment, scaling, contrast, transparent exports and target-shell rendering before release. Do not modify preserved legacy artwork as part of unrelated icon work.


## Desktop Trash raster export
The six approved Trash SVG masters remain canonical. The export script `tools/assets/render-trash-icons.sh` reuses the existing `rsvg-convert` host tool and produces eight standard raster sizes documented in `generated/README.md`. The generated assets have not yet been produced or visually validated. The renderer is developer-only, never a runtime dependency.
