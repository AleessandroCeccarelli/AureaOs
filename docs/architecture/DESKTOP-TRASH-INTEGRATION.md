<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# Desktop Trash — Integration Architecture

## Canonical artwork

The six approved SVG masters are stored under `aurea-human-interfaces/icons/source/system/trash/`. Aurea Shell embeds these exact resources at compile time and resolves one by theme and visual state. No duplicate copies belong in component directories.

## Current shell resource bridge

- `aurea-shell/src/desktop/trash.rs`: visual state contract.
- `aurea-shell/src/desktop/trash_artwork.rs`: embedded SVG lookup.
- `aurea-shell/src/desktop/mod.rs`: desktop state exposes the selected SVG bytes.

## Filesystem service

The first filesystem service lives in `aurea-desktop/src/trash/` and manages the per-user home Trash at `~/.local/share/Trash`:
- `metadata.rs`: URL-style path encoding and .trashinfo parsing/writing.
- `service.rs`: move, list, restore, permanent delete, empty, and snapshot.
- `mod.rs`: public service API.

The service records original paths and deletion timestamps in freedesktop-style .trashinfo files. Restore refuses to overwrite an existing destination. Permanent deletion is an explicit method. The visual Full state defaults to 10 retained entries and can be configured through `with_full_threshold`.

## Current limitations

This is the home Trash implementation. Cross-filesystem Trash locations, automatic filesystem watching, asynchronous shell-to-service state propagation, and user confirmation UI are not yet integrated. A failed cross-device rename returns an I/O error without leaving an orphan metadata file.

## Rendering boundary

The compositor's current render scene handles the desktop background, mapped Wayland surfaces and cursor. It does not yet contain a native desktop-icon render element or SVG rasterization path. The next rendering step must add an explicit icon element and a Vulkan-compatible texture upload/rasterization path; it must not create a fake application window.

## Safety and verification

- Restore never overwrites an existing destination.
- The service only accepts absolute existing source paths.
- Paths already inside the home Trash are rejected.
- No permanent deletion occurs during a move operation.
- Unit tests are included but have not been executed.
- No Cargo formatting, build or test command was run during this change.


## Filesystem service milestone

The home Trash service is implemented in `components/aurea-desktop/src/trash/` with metadata encoding, move/list/restore/permanent-delete/empty operations, and a configurable visual threshold (default 10 items). The shell runtime exposes a setter for the service-derived visual state.

Remaining work: instantiate and connect the service to shell runtime, add filesystem change notification and confirmation UI, implement desktop icon rasterization/texture upload and render it through the existing Vulkan scene path. Home Trash only is supported in this milestone; cross-filesystem Trash handling is pending. Tests are present but unexecuted.
