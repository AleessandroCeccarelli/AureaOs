<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Cursor Theme

Original AureaOS SVG pointer artwork and the future cursor-theme build contract for the Wayland desktop.

## Structure

- `source/` — editable SVG masters.
- `manifest.toml` — versioned asset inventory, logical sizes, output scales, hotspots and busy animation timing.
- `generated/` — reserved for Xcursor theme outputs and generated frames.
- `LICENSE.md` — project artwork license.

## Current inventory

Nine SVG masters: default, pointer, text, grab, grabbing, move, resize-nwse, resize-nesw and busy. Monochrome pointers use the approved white fill and dark outline; the busy indicator is a deliberate color exception.

## Runtime destination

The manifest is the single source of truth for source paths, hotspots, logical sizes, output scales and busy animation timing. Rasterize masters at supported logical sizes and output scales, generate timed busy frames, package standard Xcursor files and load them through the Smithay compositor cursor path. Xcursor is an asset format and does not require an X11 or XWayland runtime.

Hotspots in the manifest are expressed in the original 64 × 64 SVG coordinate space. The build pipeline must scale them together with each raster output and reject values outside the generated image bounds.

## Validation

Run the source and manifest checks from the repository root:

```sh
python3 tools/assets/validate_cursors.py
python3 -m unittest discover -s tools/assets/tests -p 'test_*.py' -v
```

The validator uses only the Python standard library. It checks manifest structure, unique cursor names, source-path containment, hotspot bounds, SVG geometry, external resource references, scripts, event handlers and animation policy. The Rust/Vulkan build does not yet consume generated Xcursor assets.

## Not yet implemented

Xcursor build pipeline, hotspot manifest, automated validation, Smithay cursor loading/state selection, fractional scaling and high-DPI review, reduced-motion fallback and pointer-state tests. SVG sources are not an installed or working cursor theme.
