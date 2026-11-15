# AureaOS Cursor Theme

Original cursor artwork for the AureaOS Wayland desktop. SVG files in source/ are editable master assets; they are not compiled Xcursor binaries and do not by themselves prove runtime integration.

## Visual contract

- Default pointer: slim, sharp white arrow with familiar desktop inclination and narrow dark outline.
- Standard interaction cursors: white artwork with the same dark outline for contrast on light and dark surfaces.
- Grab hand: palm-up and open. grabbing.svg is the closed gripping state. The compositor or shell switches states on pointer-button press/release while dragging.
- Resize cursors: corner-shaped diagonal handles, not double-headed arrows.
- Busy cursor: deliberate color exception, an eight-color radial spinner. busy.svg rotates continuously as an SVG preview/source. Runtime Xcursor packaging must rasterize timed frames; do not assume SVG animation is understood by Xcursor.

## Source inventory

| Asset | Intended role |
|---|---|
| source/default.svg | Default arrow; hotspot at (10, 5) on 64 × 64 canvas |
| source/pointer.svg | Interactive/link hand |
| source/text.svg | Text insertion |
| source/grab.svg | Open palm, ready to grab |
| source/grabbing.svg | Closed hand while dragging |
| source/move.svg | Move object/window |
| source/resize-nwse.svg | Diagonal corner resize NW–SE |
| source/resize-nesw.svg | Diagonal corner resize NE–SW |
| source/busy.svg | Multicolor continuously rotating busy indicator |

## Wayland and Xcursor packaging

Xcursor is a cursor asset file format; using it does not introduce an X11 or XWayland runtime dependency. The intended packaging pipeline is:

1. Keep these SVGs as editable source of truth.
2. Rasterize to premultiplied ARGB cursor frames at supported logical sizes and output scales.
3. Generate multiple timed busy frames from the rotating source, then package images with hotspot and animation metadata in the standard Xcursor theme layout.
4. Load the theme through the Wayland/Smithay cursor path and select states from real input, drag and operation state.
5. Test light/dark surfaces, fractional scaling, hotspot alignment, animation timing, reduced motion and pointer state transitions.

This directory currently contains source artwork only: no Xcursor binaries, compositor wiring, build pipeline or runtime behavior is claimed.

## Accessibility and behavior

- Provide a reduced-motion fallback for busy (single clear frame or non-animated progress indication).
- Never leave busy active after an operation completes or fails.
- Keep cursor size and hotspot stable across state transitions.
- Use the same white fill and dark outline for monochrome cursors.
- Do not use cursor motion as the only indication of operation status.

## Licensing and provenance

All artwork is original AureaOS project artwork, authored for this repository. No Windows, Apple, Folder11 or other third-party cursor files are included. See LICENSE.md.
