<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Human Interfaces — Audit and Completion Roadmap

**Audit baseline:** GitHub `main`, 15 November 2026  
**Purpose:** Track the design foundation, its gaps and the next controlled work.

This is a repository and specification audit. It is not evidence that prototypes or runtime features have been tested on hardware.

## 1. Product direction — authoritative

AureaOS follows **familiarity first; refinement through integration**.

- **Windows is the primary usability reference:** desktop mental model, launcher, menus, file manager, context menus, window controls, keyboard shortcuts and common workflows.
- **KDE Plasma is a secondary reference:** floating bottom dock, adaptable panel behavior and workspace organization.
- **macOS is a refinement reference:** cross-application consistency, visual finish, coherent surfaces and notification quality.

These are references for conventions and usability, not artwork or interface templates to copy. AureaOS owns its branding and visual assets. It must not make ordinary desktop tasks unfamiliar merely to appear original.

A first-time Windows user should recognize where to launch applications, find files, access context menus, manage windows and use common shortcuts.

## 2. Current inventory

| Area | Current role | Audit note |
|---|---|---|
| `assets/` | Source artwork and design inputs | Avoid duplicating runtime-ready resources |
| `brand/` | AureaOS identity | Brand usage rules and canonical logo package remain to be completed |
| `components/` | Practical examples | Keep examples small; specifications live in design-system |
| `design-system/` | Tokens, components, patterns and keyboard conventions | Canonical shared design source |
| `fonts/` | Font sources and notices | Final system font is not yet approved |
| `icons/` | Original system and application artwork | Inventory, export rules and coverage review remain |
| `previews/` | Review samples | Gallery still needs useful previews |
| `prototypes/` | Interactive and static design experiments | Simulated actions are not runtime functionality |
| `resources/` | Reusable wallpapers, illustrations, textures and sound resources | Sound specifications exist; production audio remains future work |
| `symbols/` | Curated third-party reference subset | Not a complete AureaOS icon API |
| `themes/` | Compatibility entry points | Must not become a second token source |

## 3. Work completed in this direction

- [x] Established Windows-first familiarity as the primary usability rule.
- [x] Defined KDE Plasma's floating dock as a secondary desktop influence.
- [x] Defined macOS as a reference for visual consistency and notification refinement.
- [x] Reworked the desktop specification around familiar launcher, taskbar, window, file-manager and shortcut behavior.
- [x] Reworked application guidelines around stable menu categories and command placement.
- [x] Added a shared keyboard and command convention document.
- [x] Clarified that prototypes are visual/interaction references, not proof of runtime behavior.
- [x] Made `design-system/tokens/index.css` the single token source; light/dark CSS files are compatibility imports.

## 4. Remaining design gaps

### Desktop and application behavior
- [ ] Validate the launcher and dock against the selected prototype.
- [ ] Specify exact taskbar grouping, previews, pinning and minimized-window behavior.
- [ ] Complete snap layouts, overview and multi-monitor rules.
- [ ] Verify every documented shortcut against actual intended runtime behavior before implementation.
- [ ] Complete Fides navigation, file operations, drives and context-menu behavior.

### Shared design system
- [ ] Review all component specifications against familiar desktop conventions.
- [ ] Complete typography scale, line heights, font fallback and localization guidance.
- [ ] Complete accessibility and reduced-transparency guidance.
- [ ] Define responsive and display-scaling rules.
- [ ] Add light/dark component previews.

### Visual assets
- [ ] Inventory original AureaOS icons and folder variants.
- [ ] Verify source and licensing for third-party symbols.
- [ ] Define icon sizes, optical alignment, stroke/fill and state rules.
- [ ] Complete wallpaper catalog, aspect ratios and attribution.
- [ ] Approve the production font only after repository font/license review.
- [ ] Produce original system sounds and document event mapping.

### Review and implementation
- [ ] Add focused component examples and useful preview galleries.
- [ ] Map approved patterns to Rust/native implementation owners.
- [ ] Keep design approval separate from runtime implementation status.
- [ ] Validate the native shell against the prototype and specification on actual hardware.

## 5. Directory ownership

- `assets/`: source artwork and design inputs.
- `brand/`: logo and identity rules.
- `components/`: practical component examples.
- `design-system/`: semantic tokens, component contracts, patterns and keyboard rules.
- `fonts/`: source fonts, licenses and approved font assets.
- `icons/`: original AureaOS app and system icon artwork.
- `previews/`: reviewed visual samples.
- `prototypes/`: experiments and concept validation.
- `resources/`: runtime-ready wallpapers, illustrations, textures and sounds.
- `symbols/`: curated third-party source symbols and attribution.
- `themes/`: compatibility entry points only.

Do not create parallel folders or duplicate canonical assets.

## 6. Working rules

- One file, one responsibility; keep specifications focused.
- Familiar behavior takes priority over novelty.
- Do not relocate or rename common controls without a demonstrated usability benefit.
- Do not create a second menu, shortcut, theme or component system.
- Use shared tokens; never fork palettes inside applications.
- Review both themes and accessibility for every shared component.
- Do not create placeholder assets that imply completion.
- Preserve third-party license notices and source attribution.
- Never claim simulated prototype actions are implemented system functionality.

## 7. Next sequence

1. Finish the behavioral contract for dock, windows, menus, shortcuts and Fides.
2. Review the current prototypes and align them with that contract rather than designing a competing desktop.
3. Complete the shared visual component rules and light/dark previews.
4. Audit iconography and folder artwork.
5. Complete typography, accessibility and responsive rules.
6. Move to wallpaper and sound production only after the interaction and visual system are coherent.

**Conclusion:** AureaOS should feel familiar on first use. Its identity comes from the quality of the integration, the consistency of its surfaces and its original assets—not from making users relearn their desktop.
