<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Cursor Theme

Original AureaOS SVG pointer artwork and the future cursor-theme build contract for the Wayland desktop.

## Structure

- `source/` — editable SVG masters.
- `generated/` — reserved for Xcursor theme outputs and generated frames.
- `LICENSE.md` — project artwork license.

## Current inventory

Nine SVG masters: default, pointer, text, grab, grabbing, move, resize-nwse, resize-nesw and busy. Monochrome pointers use the approved white fill and dark outline; the busy indicator is a deliberate color exception.

## Runtime destination

Rasterize masters at supported logical sizes and output scales, generate timed busy frames, package standard Xcursor files and load them through the Smithay compositor cursor path. Xcursor is an asset format and does not require an X11 or XWayland runtime.

## Not yet implemented

Xcursor build pipeline, hotspot manifest, automated validation, Smithay cursor loading/state selection, fractional scaling and high-DPI review, reduced-motion fallback and pointer-state tests. SVG sources are not an installed or working cursor theme.
