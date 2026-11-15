<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Human Interfaces

The design authority for AureaOS visual identity, desktop interaction, shared interface rules and reusable human-facing assets.

## Product principle

**Familiarity first; refinement through integration.**

- Windows is the primary usability reference for common desktop conventions, navigation, file management, menus, window controls and keyboard behavior.
- KDE Plasma informs the floating bottom dock, desktop flexibility and workspace organization.
- macOS informs cross-application consistency, visual finish and notification quality.

These are usability references, not artwork or interfaces to copy. AureaOS owns its identity and implementation.

## Directory map

| Directory | Responsibility |
|---|---|
| `brand/` | AureaOS identity, logo masters and presentation artwork |
| `design-system/` | Canonical tokens, component contracts, patterns and keyboard conventions |
| `icons/` | Original application, system and folder artwork |
| `symbols/` | Curated third-party symbol sources and AureaOS semantic catalog |
| `cursors/` | Original pointer artwork and future cursor-theme outputs |
| `fonts/` | Font sources, notices and future Aurea Sans family |
| `themes/` | Compatibility entry points delegating to canonical tokens |
| `resources/` | Wallpapers, illustrations, textures and sound-theme assets |
| `components/` | Small visual examples of shared components |
| `prototypes/` | Desktop and application design experiments |
| `previews/` | Review exports for components, fonts, icons and themes |

There is no parallel `assets/` directory: reusable assets have one canonical home.

## Source-of-truth rules

1. `design-system/tokens/index.css` is the sole canonical CSS token source.
2. Specifications define intended behavior; they do not prove runtime implementation.
3. Editable masters remain in their relevant `source/` collection.
4. Generated/runtime outputs stay separate from masters.
5. Third-party source assets retain their upstream licenses and notices.
6. Never replace or silently modify a source when generating an output.
7. Every collection documents purpose, provenance, format, license and integration status.

## Main specifications

- [Application design guidelines](APP-DESIGN-GUIDELINES.md)
- [Desktop experience](DESKTOP-EXPERIENCE-SPEC.md)
- [Design system](design-system/README.md)
- [Keyboard and command conventions](design-system/KEYBOARD-AND-COMMAND-CONVENTIONS.md)
- [Icons](icons/README.md) · [Symbols](symbols/README.md) · [Cursors](cursors/README.md)
- [Fonts](fonts/README.md) · [Resources](resources/README.md)
- [Licensing and provenance](LICENSING-AND-PROVENANCE.md)
- [Completion roadmap](AUDIT-AND-COMPLETION-ROADMAP.md)

## Licensing

Original AureaOS material is GPL-3.0-or-later unless a file-specific notice says otherwise. Third-party material is not relicensed by the project. See [Licensing and Provenance](LICENSING-AND-PROVENANCE.md), repository `LICENSE` and `licenses/THIRD_PARTY_NOTICES.md`.

## Status

Some collections are approved source artwork but are not yet built, packaged, connected to the shell or validated on hardware. The roadmap distinguishes these states.
