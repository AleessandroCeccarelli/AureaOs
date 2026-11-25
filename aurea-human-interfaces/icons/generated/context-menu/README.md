# Desktop context-menu raster assets

Canonical editable masters:
- `aurea-human-interfaces/icons/source/system/context-menu/light.svg`
- `aurea-human-interfaces/icons/source/system/context-menu/dark.svg`

Generate the two RGBA8 runtime assets from the repository root:

```bash
bash tools/assets/render-context-menu.sh
```

The host requires `rsvg-convert`, Python 3 and Noto Sans:
- Debian 13: `sudo apt install librsvg2-bin fonts-noto-core`
- Fedora: `sudo dnf install librsvg2-tools google-noto-sans-fonts`

Noto Sans is used only while rasterizing the SVG masters. The compositor consumes raw RGBA8 data and does not link a font, SVG, Cairo, GLib or Pango runtime.

The texture canvas is 256 × 256 pixels. The visible menu occupies the top-left 248 × 160 pixels; the remaining area is transparent so the existing bounded square desktop-artwork upload contract can be reused. Generated binaries must be visually inspected on Fedora before they are connected to the frame renderer.
