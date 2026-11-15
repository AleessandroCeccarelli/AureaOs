<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Icon Library

Owns original AureaOS application, system and Fides folder artwork.

## Structure

- `source/system/` — system and first-party application marks.
- `source/folders/` — original folder/location artwork, including approved v5.
- `source/applications/` — application icon masters, with complete-tile and standalone-symbol variants.
- `generated/` — future exports and runtime packages.
- `FOLDER-ICON-SPEC.md` — folder artwork contract.

Brand presentation artwork belongs in `../brand/`; generic third-party symbols belong in `../symbols/`.

## Current state

System sources include AureaOS icon and symbol artwork, Aurea AI Orb, File Manager and Wallet. Folder v5 contains 14 semantic locations in light and dark variants, open/drop states and an SVG sprite. First-party application artwork now includes the approved Quaero identity in two coordinated SVG variants: a complete icon tile and a transparent standalone symbol. Other first-party application icons remain to be designed.

## Rules and destination

Keep editable masters in `source/`, generated sizes in `generated/`. Each first-party application must provide a complete tile for launchers/catalogs and a matching standalone symbol for the dock and compact UI. Define optical weight, alignment, geometry, states, transparent exports, supported sizes and packaging. The intended library will cover shell, settings, Fides, Quaero, Aurea Wallet, Aurea AI and future first-party applications. Runtime API and visual validation remain future work.
