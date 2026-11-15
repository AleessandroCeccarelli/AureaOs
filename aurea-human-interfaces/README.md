<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Human Interfaces

The single design authority for AureaOS visual identity, desktop interaction, shared interface rules and reusable human-facing assets.

## Product direction

**Familiarity first; refinement through integration.**

- **Windows** is the primary usability reference for desktop conventions, navigation, file management, menus, window controls and keyboard behavior.
- **KDE Plasma** informs the floating bottom dock, desktop flexibility and workspace organization.
- **macOS** informs cross-application consistency, visual finish and notification refinement.

These are usability references, not artwork or interfaces to copy. AureaOS owns its identity, visual language and implementation.

## Design authority

The following hierarchy prevents competing specifications:

1. `design-system/tokens/index.css` is the sole canonical CSS token source.
2. [Desktop Experience](DESKTOP-EXPERIENCE-SPEC.md) defines the overall shell experience and desktop behavior.
3. [Application Design Guidelines](APP-DESIGN-GUIDELINES.md) defines shared first-party application conventions.
4. `design-system/components/` defines reusable component contracts.
5. `design-system/patterns/` defines recurring compositions and workflows.
6. [Keyboard and Command Conventions](design-system/KEYBOARD-AND-COMMAND-CONVENTIONS.md) owns shortcut defaults, ownership and command consistency.
7. Collection READMEs define asset-specific formats, provenance, licensing and integration status.

If two documents appear to conflict, preserve the hierarchy above and update the lower-level document rather than creating a second design rule.

## Directory map

| Directory | Responsibility |
|---|---|
| `brand/` | AureaOS identity references, future canonical logo masters and presentation artwork |
| `design-system/` | Canonical tokens, component contracts, patterns and keyboard conventions |
| `icons/` | Original application, system, launcher and folder artwork |
| `symbols/` | Curated third-party symbol sources and AureaOS semantic catalog |
| `cursors/` | Original pointer artwork and future Xcursor outputs |
| `fonts/` | Font sources, notices and typography decisions |
| `themes/` | Compatibility entry points delegating to canonical tokens |
| `resources/` | Wallpapers, illustrations, textures and sound-theme assets |
| `components/` | Focused visual examples of shared components |
| `prototypes/` | Desktop and application design experiments |
| `previews/` | Review exports for components, fonts, icons and themes |

There is no parallel `assets/` directory: reusable assets have one canonical home.

## Current asset inventory

- **Application icons:** coordinated complete-tile and transparent-symbol SVG variants exist for Quaero, Fides, Aurea Wallet, Aurea Store and Aurea Time.
- **Launcher identity:** dedicated complete-tile and transparent-symbol SVG variants exist under `icons/source/system/`.
- **Folder artwork:** Fides folder family v5 provides 14 semantic locations, light/dark variants and open/drop states.
- **System symbols:** a curated Fluent UI subset contains 89 concepts and 178 SVG variants.
- **Sounds:** 16 original procedural source effects and a review preview are retained; runtime event files are a separate pending integration step.
- **Fonts:** Noto families and Source Code Pro are retained as temporary development support. The proprietary Aurea Sans project is explicitly deferred and is not a current milestone.
- **Cursors:** nine original SVG masters are retained; Xcursor generation and compositor integration remain pending.
- **Wallpapers and illustrations:** the current collection contains nine wallpapers and three original SVG illustrations.
- **Themes:** canonical light/dark CSS tokens and compatibility wrappers are present.

These are repository source inventories, not claims that every asset is installed, packaged, rendered by the shell or validated on hardware.

## Source-of-truth rules

1. `design-system/tokens/index.css` is the sole canonical CSS token source.
2. Specifications define intended behavior; they do not prove runtime implementation.
3. Editable masters remain in their relevant `source/` collection.
4. Generated/runtime outputs stay separate from masters.
5. Third-party source assets retain their upstream licenses and notices.
6. Never replace or silently modify a source when generating an output.
7. Every collection documents purpose, provenance, format, license and integration status.
8. Do not create duplicate assets or parallel palettes to solve a local presentation issue.
9. Add new artwork only for an identified product or system need.

## Main specifications

- [Desktop experience](DESKTOP-EXPERIENCE-SPEC.md)
- [Application design guidelines](APP-DESIGN-GUIDELINES.md)
- [Design system](design-system/README.md)
- [Keyboard and command conventions](design-system/KEYBOARD-AND-COMMAND-CONVENTIONS.md)
- [Icons](icons/README.md) · [Symbols](symbols/README.md) · [Cursors](cursors/README.md)
- [Fonts](fonts/README.md) · [Resources](resources/README.md)
- [Licensing and provenance](LICENSING-AND-PROVENANCE.md)
- [Status and completion roadmap](AUDIT-AND-COMPLETION-ROADMAP.md)

## Licensing

Original AureaOS material is GPL-3.0-or-later unless a file-specific notice says otherwise. Third-party material is not relicensed by the project. See [Licensing and Provenance](LICENSING-AND-PROVENANCE.md), repository `LICENSE` and `licenses/THIRD_PARTY_NOTICES.md`.

## Status boundary

Human Interfaces currently contains design specifications, source artwork, CSS tokens, prototypes and third-party reference assets. Native shared components, asset build pipelines, runtime packaging, shell integration and target-hardware validation are separate implementation tasks. Never report a source asset or prototype as a working runtime feature.
