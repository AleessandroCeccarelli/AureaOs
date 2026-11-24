<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Generated Icon Assets

Destination for reproducible icon exports and runtime packages derived from `../source/`.

Never edit generated files as masters. Record source version, build command/tool version, output sizes and validation results.

## Desktop Trash RGBA8 export

- Canonical sources: `../source/system/trash/{light,dark}/trash-{empty,half,full}.svg`.
- Generator: `tools/assets/render-trash-icons.sh`.
- SVG renderer: existing host-only `rsvg-convert`, shared with cursor-theme generation.
- PNG conversion: `tools/assets/png_to_rgba.py`, using only Python standard-library `zlib` and `struct`.
- Runtime output: `trash/trash-{light,dark}-{empty,half,full}-{size}.rgba`.
- Sizes: 64, 96, 128 and 192 pixels, covering 1x, 1.5x, 2x and 3x rendering for a 64-logical-pixel icon.
- Raw format: tightly packed, row-major RGBA8, no row padding; expected byte count is width × height × 4.
- The converter accepts only non-interlaced 8-bit RGBA PNGs up to 256 × 256 pixels, validates PNG chunk CRCs and decompression length, and supports PNG filters 0–4.
- The script records the renderer version and verifies each raw asset's exact byte length.

Run from any working directory:

```sh
bash tools/assets/render-trash-icons.sh
```

Host tools:

- Debian 13: `sudo apt install librsvg2-bin`
- Fedora: `sudo dnf install librsvg2-tools`
- Python 3 is required; the converter has no third-party Python package dependency.

This avoids a PNG decoder in the compositor runtime and reuses the SVG renderer already required by the cursor asset pipeline. Neither `rsvg-convert` nor librsvg is an AureaOS runtime dependency. The export pipeline has not yet been executed; generated assets must be visually inspected before compositor integration.
