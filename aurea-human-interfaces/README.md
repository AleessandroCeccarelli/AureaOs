# Aurea Human Interfaces

Aurea Human Interfaces (AHI) is the planned shared visual identity and interface library for AureaOS. It will help Aurea Desktop and first-party applications use consistent typography, icons, themes, symbols, and reusable interface components.

This repository area is an organizational structure only. It contains no production assets or implementation. The existing principles and component requirements remain documented in [`docs/human-interfaces/`](../docs/human-interfaces/); this README describes where future materials may live and how to organize them. Folder names and file formats below are conventions proposed for discussion, not approved implementation choices.

## Directory map

| Directory | Purpose and expected contents | Why it is separate |
| --- | --- | --- |
| `fonts/` | Typeface source files, font metadata, and any optimized distribution files. | Keeps typography assets and their licensing/provenance information together. |
| `icons/` | AureaOS brand, application, and system icon artwork, grouped by role. | Makes it easier to find icons and keep distinct icon families consistent. |
| `symbols/` | Shared interface symbols and any project-authored emoji artwork or symbol mapping documentation. | Separates small semantic glyphs from full application/brand icons. |
| `themes/` | Shared design tokens and theme-specific values for light and dark appearances. | Provides one place for shared colors and other visual variables rather than per-app copies. |
| `resources/` | Supporting visual materials such as wallpapers, illustrations, and textures. | Keeps larger contextual artwork separate from interface controls and icons. |
| `components/` | Component specifications, state references, and reusable visual examples. | Organizes shared interface patterns without implying that a code library exists yet. |
| `previews/` | Review images and assembled overviews of fonts, icons, themes, and components. | Collects convenient visual review exports without treating them as editable originals. |
| `prototypes/` | Early static or interactive explorations used to assess how elements work together. | Keeps exploratory work distinct from approved design references and production assets. |

### Proposed subfolders

```text
aurea-human-interfaces/
├── fonts/
│   ├── source/
│   └── generated/
├── icons/
│   ├── source/
│   │   ├── applications/
│   │   ├── brand/
│   │   └── system/
│   └── generated/
├── symbols/
│   ├── source/
│   └── generated/
├── themes/
│   ├── tokens/
│   ├── light/
│   └── dark/
├── resources/
│   ├── wallpapers/
│   ├── illustrations/
│   └── textures/
├── components/
│   ├── specifications/
│   └── examples/
├── previews/
│   ├── fonts/
│   ├── icons/
│   ├── themes/
│   └── components/
└── prototypes/
    ├── static/
    └── interactive/
```

Subfolders can be expanded when there is real content to organize. In particular, `generated/` is reserved for derived outputs if a future workflow needs them; no generation tool or build process is selected by this structure.

## File formats

These are suggested formats, not requirements. Choose a format that preserves the editable source and suits the intended consumer.

| Content | Suggested editable/source formats | Possible generated or review formats |
| --- | --- | --- |
| Fonts | Original licensed font source where available; `UFO` source directories; `designspace` / `glyphs` files where applicable; `OTF` / `TTF` when those are the authored font files. | Optimized or subset `OTF` / `TTF`; `WOFF2` only if a web preview needs it. Keep source and distribution copies distinct. |
| Icons and symbols | `SVG` for vector artwork; `JSON` or `TOML` for mappings/metadata when useful; `MD` for guidance. | `PNG` for raster previews or size-specific exports; other runtime formats only after the consuming technology is chosen. |
| Themes and tokens | `JSON`, `TOML`, or `CSS` custom properties, selected when implementation needs are known; `MD` for explanation. | Compiled token files only if a future integration requires them; identify them as generated. |
| Resources | Editable artwork in its native source format where practical; `SVG` for scalable illustrations; `PNG`, `JPEG`, or `WebP` for raster artwork. | Optimized raster copies for distribution or previews. Preserve attribution and license notes. |
| Component specifications | `MD` for behavior and accessibility requirements; `SVG` or `PNG` for diagrams and state illustrations; `JSON` only for structured examples when helpful. | `PNG` or `PDF` review sheets. |
| Previews | Usually not an editable source; keep links or notes to the source material in `MD`. | `PNG`, `JPEG`, or `WebP` images; `PDF` overview sheets. |
| Prototypes | `MD`, `SVG`, and `PNG` for static explorations. Interactive format is intentionally undecided; document its format and how to view it before adding one. | Exported `PNG` or `PDF` snapshots for review. |

The project must confirm font, icon, image, and other third-party licenses before redistribution. Record provenance and license information alongside contributed material, for example in a nearby `README.md` or a future asset inventory. Do not treat generated files as the editable source of truth.

## Naming and organization conventions

- Use lowercase English names and hyphens between words for directories and filenames, for example `application-grid.svg` or `dark-theme.md`.
- Use descriptive, role-based names. Avoid names such as `final`, `final-v2`, or unexplained abbreviations.
- Keep each asset in the category matching its primary purpose. Do not copy the same source into multiple categories; link to it from documentation or previews instead.
- Keep editable originals under `source/` where that distinction is useful. Put derived exports under `generated/` and never edit those by hand.
- Add dimensions or scale suffixes only when they distinguish required variants, for example `brand-mark-16.svg` only if a size-specific design is actually needed.
- For theme values, prefer semantic names (such as a surface or text role) over names tied to a particular screen or component. Exact token naming remains to be defined.
- Store brief usage, provenance, and license notes close to assets when they would otherwise be unclear.
- Use Markdown (`.md`) for explanatory notes, SVG (`.svg`) for editable vector graphics, and raster formats such as PNG or WebP for image exports. Use JSON or TOML only for structured data that needs machine reading; the project has not selected a token schema.

## Design alignment

Future contents should follow the existing principles: clarity, consistency, contextual layouts, accessibility, and restrained visual effects. Light and dark appearances should both preserve readable contrast; keyboard focus and reduced motion must be considered. AureaOS Human Interfaces is responsible for presentation and interaction patterns, while the AureaOS System API is a separate planned concern for system capabilities.

See [`docs/human-interfaces/README.md`](../docs/human-interfaces/README.md), [`DESIGN-PRINCIPLES.md`](../docs/human-interfaces/DESIGN-PRINCIPLES.md), and [`COMPONENTS.md`](../docs/human-interfaces/COMPONENTS.md) for the design direction and intended behavior.
