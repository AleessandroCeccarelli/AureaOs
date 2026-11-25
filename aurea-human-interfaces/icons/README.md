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




## Desktop Trash RGBA8 export
The six approved SVG masters remain canonical. `tools/assets/render-trash-icons.sh` uses the existing `rsvg-convert` host tool and `tools/assets/png_to_rgba.py` (Python standard library only) to generate tightly packed RGBA8 assets at 64, 96, 128 and 192 pixels. These correspond to 1x through 3x output scale for a 64-logical-pixel icon. The export has not yet been executed or visually validated; no runtime SVG or PNG decoder is introduced.


## Desktop context-menu artwork

- Canonical masters: source/system/context-menu/light.svg and source/system/context-menu/dark.svg.
- Typography: Noto Sans Variable, Google Fonts / The Noto Project, SIL OFL 1.1. The canonical font is retained at `../fonts/source/google-fonts/noto-sans/NotoSans-Variable.ttf`; the export script uses this repository copy and does not require a host-installed font.
- Export: bash tools/assets/render-context-menu.sh; generated outputs belong in generated/context-menu/.
- The 256 × 256 RGBA8 canvas contains a 248 × 160 visible menu panel at its top-left. The transparent remainder allows reuse of the existing bounded square artwork upload contract.
- The compositor now resolves these generated RGBA files from the Human Interfaces asset root, caches the selected theme texture and draws the menu after Wayland surfaces but before the pointer. Export and visual validation on Fedora are still required before calling the menu verified.
