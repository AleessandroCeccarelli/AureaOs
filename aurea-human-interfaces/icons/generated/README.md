<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Generated Icon Assets

Destination for reproducible icon exports and runtime packages derived from `../source/`.

Never edit generated files as masters. Record source version, build command/tool version, output sizes and validation results.

## Desktop Trash export

- Source masters: `../source/system/trash/{light,dark}/trash-{empty,half,full}.svg`.
- Generator: `tools/assets/render-trash-icons.sh`.
- Renderer: the existing `rsvg-convert` host tool, also used by the cursor-theme asset pipeline.
- Outputs: `trash/trash-{light,dark}-{empty,half,full}-{size}.png`.
- Sizes: 16, 24, 32, 48, 64, 96, 128 and 256 pixels.
- The script records the exact renderer version in `trash/EXPORT-MANIFEST.txt` and checks file presence, PNG signature, exact square dimensions and 8-bit RGBA format.
- The source SVGs have transparent canvases; transparency appearance still requires visual inspection.

Run from any working directory with:

```sh
bash tools/assets/render-trash-icons.sh
```

Install the host-only tool when needed:

- Debian 13: `sudo apt install librsvg2-bin`
- Fedora: `sudo dnf install librsvg2-tools`

This reuses the renderer already required by the cursor-theme asset builder. Neither `rsvg-convert` nor librsvg is an AureaOS runtime dependency. Generated exports have not yet been produced or visually validated; do not wire them into the compositor until transparency, theme contrast and optical alignment have been inspected.
