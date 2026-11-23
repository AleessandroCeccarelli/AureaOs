<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Desktop Items and Visual Stacks

## Product rules

- A new user desktop contains only the system Trash entry.
- Users may add files, folders and application launchers to their desktop.
- Desktop entries have stable identifiers and user-controlled logical positions.
- Stack grouping is opt-in and disabled by default.
- A stack is a derived visual grouping of desktop file entries. It is not a directory, archive or filesystem operation.
- Grouping must never move, rename, copy, delete or rewrite source files.
- Files that do not qualify for a group, and groups containing only one file, remain individual desktop entries.
- Users may exclude individual entries from stack grouping.
- Disabling grouping restores the individual entries without changing their identity or position.
- The Trash is a system entry and cannot be removed through ordinary desktop-item operations.

## Grouping modes

- **By category:** Images, Documents, Video, Audio, Archives and Other files.
- **By extension:** separate groups such as .jpg, .png, .pdf and .mp4.

Only file entries participate in stacks. Folders, application launchers and Trash remain individual entries.

## Geometry and input contract

Desktop item positions and pointer hit-testing use global desktop logical coordinates. The API geometry types do not depend on a monitor's physical resolution, output scale, Smithay, or Vulkan.

- DesktopRect uses a top-left origin, includes its left/top edges and excludes its right/bottom edges.
- Hit-region dimensions are supplied by the presentation layer in logical units.
- DesktopHitTest resolves overlapping regions deterministically: the last item in the supplied visual order is considered on top.
- A zero-width or zero-height region never produces a hit.
- Rendering, selection visuals, pointer capture, drag-and-drop and activation remain responsibilities of the shell/compositor integration; this geometry contract does not implement those interactions.

This separation allows the same desktop model to work across outputs with different scale factors. Physical pixel conversion belongs at the output/render boundary, not in the shared desktop API.

## Architecture boundary

The module components/aurea-desktop-api/src/desktop/ owns technology-independent data contracts and deterministic grouping rules. It has no dependency on Smithay, Wayland, Vulkan, a file manager or a filesystem watcher.

The shell will own presentation, interaction and user preferences. The compositor will own rendering and input hit-testing, not file discovery or filesystem mutation. A future desktop service/Fides integration must provide validated desktop entries and observe filesystem changes through an explicit service boundary.

This initial implementation is an in-memory model. Persistence, file discovery, MIME database integration, drag-and-drop, context menus, pile expansion UI, live filesystem updates and actual rendering are not claimed as implemented.
