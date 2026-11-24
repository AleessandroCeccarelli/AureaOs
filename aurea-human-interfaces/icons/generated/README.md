<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Generated Icon Assets

Destination for reproducible icon exports and runtime packages derived from `../source/`.

Never edit generated files as masters. Record source version, build command/tool version, output sizes and validation results.

## Desktop Trash export

- Source masters: `../source/system/trash/{light,dark}/trash-{empty,half,full}.svg`.
- Generator: `tools/assets/render-trash-icons.sh`.
- Renderer: `resvg 0.45.1`, used only on the developer/asset-generation host.
- Outputs: `trash/trash-{light,dark}-{empty,half,full}-{size}.png`.
- Sizes: 16, 24, 32, 48, 64, 96, 128 and 256 pixels.
- Output is transparent PNG; the script checks file presence and the PNG signature.

Run from any working directory with:

```sh
bash tools/assets/render-trash-icons.sh
```

The script refuses a different renderer version to keep generated assets reproducible. Debian 13 Trixie provides `resvg 0.45.1-1` in the `resvg` package. Fedora package availability is not confirmed; the script documents a version-pinned Cargo installation route. The renderer is not a runtime dependency of AureaOS.

The export pipeline is committed but has not yet been executed or visually validated. Do not wire these files into the compositor until output dimensions, transparency, theme contrast and optical alignment have been inspected.
