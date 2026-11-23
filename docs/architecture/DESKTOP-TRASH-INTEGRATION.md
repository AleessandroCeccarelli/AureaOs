<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# Desktop Trash — Shell Integration

## Scope

Aurea Shell embeds the six approved SVG masters at compile time and exposes the selected asset through the platform-independent desktop state. The Fides folder/location artwork is separate and is not used for the desktop Trash.

## Asset matrix

| Appearance | Empty | Partially full | Full |
|---|---|---|---|
| Light | `light/trash-empty.svg` | `light/trash-half.svg` | `light/trash-full.svg` |
| Dark | `dark/trash-empty.svg` | `dark/trash-half.svg` | `dark/trash-full.svg` |

The artwork has a transparent canvas. Do not bake desktop wallpaper or a background tile into the icon.

## Ownership and responsibilities

- `desktop/trash.rs`: semantic content states only.
- `desktop/trash_artwork.rs`: compile-time resource embedding and theme/state lookup only.
- `desktop/mod.rs`: aggregate desktop presentation state and expose the selected SVG bytes.
- `aurea-human-interfaces/icons/source/system/trash/`: canonical editable SVG masters.

SVG bytes are static and owned by the shell binary. A presentation backend must decode and render them; this resource resolver does not itself draw pixels or open a desktop window.

## State contract

`TrashContentsState` has three states: Empty, PartiallyFull and Full. It defaults to Empty. The Trash service must update this state when its contents change by calling `DesktopState::set_trash_contents` (or the equivalent runtime state update).

The exact count thresholds, file discovery, restore/delete operations, permanent-delete confirmation and filesystem monitoring belong to the future Trash service. They are intentionally not guessed or implemented by this artwork integration.

## Theme contract

`DesktopTheme` selects Light or Dark artwork. Changing the theme changes only the material palette; changing Trash contents changes only the depicted contents.

## Validation status

Unit tests are included for the six compile-time mappings, UTF-8 SVG structure, theme selection and state selection. They have **not** been executed as part of this change. No renderer integration or runtime filesystem watcher is claimed.
