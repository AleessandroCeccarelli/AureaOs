<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Cursor Theme

Original AureaOS SVG pointer artwork and the future cursor-theme build contract for the Wayland desktop.

## Structure

- `source/` — editable SVG masters.
- `manifest.toml` — versioned asset inventory, logical sizes, output scales, hotspots and busy animation timing.
- `generated/` — reserved for Xcursor theme outputs and generated frames.
- `LICENSE.md` — project artwork license.

## Current inventory

Twenty-six SVG masters cover normal and alternate selection, link, text and vertical text, precision, help, unavailable, copy, alias, pin, person, open/closed hand, move/all-scroll, horizontal/vertical/diagonal resize, handwriting, busy and working-in-background, zoom and contextual-menu feedback. Both high-contrast palettes are provided: light pointers (white fill, dark outline) and dark pointers (dark fill, white outline). Windows also offers an inverted pointer that adapts to the pixels beneath it and custom colors; these are accessibility rendering modes, not extra static cursor themes, and remain a future compositor-level feature.

## Platform role coverage

The theme must package canonical Linux/Xcursor names and compatibility aliases, including `left_ptr`, `default`, `arrow`, `hand2`, `pointer`, `xterm`, `ibeam`, `crosshair`, `help`, `watch`, `wait`, `left_ptr_watch`, `progress`, `not-allowed`, `forbidden`, `no-drop`, `copy`, `alias`, `grab`, `grabbing`, `fleur`, `all-scroll`, `ew-resize`, `ns-resize`, `nwse-resize`, `nesw-resize`, `size_hor`, `size_ver`, `size_fdiag`, `size_bdiag`, `pencil`, `openhand` and `closedhand`. Names are mapped to shared artwork where their semantics match; the theme must not invent visually different meanings for aliases.

Windows additionally distinguishes alternate select, pen, row/column sizing and activity states. macOS distinguishes copy/alias drag feedback, open/closed hand, screenshot selection and directional resize. AureaOS should implement the shared desktop roles first and expose screenshot-specific pointers only in the screenshot tool.

## Runtime destination

The manifest is the single source of truth for source paths, hotspots, logical sizes, output scales and busy animation timing. Rasterize masters at supported logical sizes and output scales, generate timed busy frames, package standard Xcursor files and load them through the Smithay compositor cursor path. Xcursor is an asset format and does not require an X11 or XWayland runtime.

Hotspots in the manifest are expressed in the original 64 × 64 SVG coordinate space. The build pipeline must scale them together with each raster output and reject values outside the generated image bounds.

## Validation

Run the source and manifest checks from the repository root:

```sh
python3 tools/assets/validate_cursors.py
python3 -m unittest discover -s tools/assets/tests -p 'test_*.py' -v
```

The validator uses only the Python standard library. It checks manifest structure, unique cursor names, source-path containment, hotspot bounds, SVG geometry, external resource references, scripts, event handlers and animation policy. Build the light theme after validation with `python3 tools/assets/build_cursor_theme.py --output aurea-human-interfaces/cursors/generated/AureaOS-Light`. Build the dark theme with `python3 tools/assets/build_cursor_theme.py --manifest aurea-human-interfaces/cursors/manifest-dark.toml --output aurea-human-interfaces/cursors/generated/AureaOS-Dark`. The build requires `rsvg-convert` (Fedora: `librsvg2-tools`; Debian: `librsvg2-bin`) and `xcursorgen`. Each build writes to the explicitly selected output directory; the builder refuses to overwrite an existing theme. Move the old output aside before rebuilding. GitHub Actions builds and publishes the generated theme as a workflow artifact.

The builder emits standard Xcursor names plus common Linux/toolkit aliases as symlinks to the canonical images. The compositor now normalizes Smithay/W3C cursor icon names and Linux/Xcursor aliases to the 26 canonical AureaOS asset names in `input/cursor_theme.rs`. This is selection plumbing only: generated Xcursor image data is not yet loaded by the runtime Vulkan renderer.

## Not yet implemented

Runtime Xcursor image loading and Vulkan texture selection, fractional scaling and high-DPI review, reduced-motion fallback, animated busy-frame scheduling and full pointer-state tests. Smithay cursor-name normalization is implemented, but the native Vulkan path still renders its built-in arrow for every named cursor. SVG masters and manifests remain source assets until the Xcursor packages are installed and the runtime loader is integrated.
