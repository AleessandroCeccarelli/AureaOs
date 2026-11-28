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

The validator uses only the Python standard library. It checks manifest structure, unique cursor names, source-path containment, hotspot bounds, SVG geometry, external resource references, scripts, event handlers and animation policy. Build the light theme after validation with `python3 tools/assets/build_cursor_theme.py --output aurea-human-interfaces/cursors/generated/AureaOS-Light`. Build the dark theme with `python3 tools/assets/build_cursor_theme.py --manifest aurea-human-interfaces/cursors/manifest-dark.toml --output aurea-human-interfaces/cursors/generated/AureaOS-Dark`. The build requires `rsvg-convert` (Fedora: `librsvg2-tools`; Debian: `librsvg2-bin`) and `xcursorgen` (Debian: provided by `x11-apps`). In the ISO pipeline these tools run only in the disposable builder; only generated Xcursor files enter the target image. Each build writes to the explicitly selected output directory; the builder refuses to overwrite an existing theme. Move the old output aside before rebuilding. GitHub Actions builds and publishes the generated theme as a workflow artifact.

The builder emits standard Xcursor names plus common Linux/toolkit aliases as symlinks to the canonical images. The compositor normalizes Smithay/W3C names in `input/cursor_theme.rs`, resolves the selected `AureaOS-Light` or `AureaOS-Dark` Xcursor file, decodes the closest supported raster size, converts ARGB pixels to RGBA8 and uploads the selected frames to a per-device Vulkan texture cache. The asset hotspot and actual image dimensions drive per-output geometry. A missing theme or cursor safely falls back to the built-in arrow.

## Runtime integration and remaining verification

The compositor resolves the selected `AureaOS-Light` or `AureaOS-Dark` Xcursor file from `AUREA_CURSOR_THEME_ROOT` (default `/usr/share/icons`). A dedicated bounded worker performs file I/O and Xcursor decoding off the render thread. The render thread polls decoded results, submits the selected raster frames to Vulkan, and publishes textures only after their upload fences signal. The selected frame uses its own dimensions and hotspot.

Xcursor animation timing is read from each frame's `delay_ms`; the compositor selects the current frame from a monotonic clock. The event loop schedules 16 ms frames while the `busy` cursor is active and during a short first-use warm-up window so asynchronous decoding/upload can complete without a client input event. Missing theme files continue to fall back to the built-in arrow.

The Debian Live hook builds both palettes from the SVG masters using `rsvg-convert` and `xcursorgen`, installs them under `/usr/share/icons/AureaOS-Light` and `/usr/share/icons/AureaOS-Dark`, and validates all 26 canonical cursor files per theme. The final SquashFS validator repeats the check against the packaged image.

Still open: reduced-motion policy, physical fractional/high-DPI visual review, pointer-state tests, and a full Fedora Cargo/ISO verification. The native cursor cache is bounded to 32 assets / 16 MiB, uses least-recently-used eviction, and limits a single animated asset to 4 MiB (falling back to its first frame if larger). `left_ptr_watch` is currently a static artwork asset, so only `busy` is scheduled as an animated cursor. These source changes have not yet been built or tested.
