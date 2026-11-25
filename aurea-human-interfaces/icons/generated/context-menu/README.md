# Desktop context-menu raster assets

Canonical editable masters:
- `aurea-human-interfaces/icons/source/system/context-menu/light.svg`
- `aurea-human-interfaces/icons/source/system/context-menu/dark.svg`

Generate the two RGBA8 runtime assets from the repository root:

```bash
bash tools/assets/render-context-menu.sh
```

The host requires `rsvg-convert` and Python 3:
- Debian 13: `sudo apt install librsvg2-bin`
- Fedora: `sudo dnf install librsvg2-tools`

Noto Sans Variable is already bundled in Human Interfaces at `fonts/source/google-fonts/noto-sans/NotoSans-Variable.ttf`. The export script points Fontconfig directly at that repository file; no separate host font installation or download is required.

The compositor loads the generated RGBA8 artwork once, on first menu use, and retains its Vulkan texture in the bounded desktop-artwork cache. For a development checkout, set the asset root before starting AureaOS:

```bash
AUREA_HUMAN_INTERFACES_ROOT="$PWD/aurea-human-interfaces" <aureaos-launch-command>
```

Installed packages place the contents of this Human Interfaces directory under `/usr/share/aureaos/human-interfaces`, which is the compositor default. The root can be overridden with `AUREA_HUMAN_INTERFACES_ROOT`.

The texture canvas is 256 × 256 pixels. The visible menu occupies the top-left 248 × 160 pixels; the remaining area is transparent. PNG alpha is premultiplied by the shared converter to match the compositor's Vulkan blend state. The compositor scales the canvas for each output and clips it to that output. Generated binaries must be exported and visually inspected on Fedora before the menu can be considered validated.
