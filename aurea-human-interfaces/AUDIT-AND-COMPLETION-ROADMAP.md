<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Human Interfaces — Status and Completion Roadmap

**Baseline:** repository main, 15 November 2026. This roadmap separates approved design direction, retained source assets, pending production work and runtime integration. It does not claim that source assets or prototypes are installed or validated on hardware.

## Product direction

AureaOS follows **familiarity first; refinement through integration**.

- Windows is the primary usability reference for desktop conventions and common workflows.
- KDE Plasma informs the floating bottom dock, desktop flexibility and workspace organization.
- macOS informs cross-application consistency, visual finish and notification refinement.

These are usability references, not artwork or interface templates to copy. AureaOS owns its identity and implementation.

## Canonical design authority

The hierarchy is documented in [Human Interfaces](README.md) and [Design System](design-system/README.md):

1. design-system/tokens/index.css — sole canonical CSS token source.
2. DESKTOP-EXPERIENCE-SPEC.md — overall desktop and shell experience.
3. APP-DESIGN-GUIDELINES.md — shared first-party application conventions.
4. design-system/components/ — reusable component contracts.
5. design-system/patterns/ — recurring compositions and workflows.
6. design-system/KEYBOARD-AND-COMMAND-CONVENTIONS.md — shortcut and command ownership.
7. Collection READMEs — asset-specific formats, provenance, licensing and integration status.

## Current inventory

| Area | Present in repository | Remaining work |
|---|---|---|
| Brand | Rome-inspired identity and presentation artwork | Canonical vector logo package and formal usage guidance |
| Design system | Canonical light/dark CSS tokens, six component specifications, four interaction patterns and shortcut conventions | Review completeness, examples, previews and native implementation |
| Application icons | Quaero, Fides, Aurea Wallet, Aurea Store and Aurea Time; complete-tile and transparent-symbol variants | Export pipeline, previews and runtime integration |
| Launcher/system artwork | Dedicated AureaOS launcher tile and transparent symbol; legacy system artwork preserved | Review legacy assets separately; define system icon usage |
| Fides folders | v5 family with 14 semantic locations, light/dark variants, open/drop states and SVG sprite | Runtime loading and visual validation in Fides |
| Symbols | Curated Fluent UI subset: 89 concepts and 178 SVG variants | Verify semantic mappings; define API, packaging, fallback and loader |
| Cursors | Nine original SVG masters | Xcursor generation, hotspot metadata, animation frames and Smithay integration |
| Fonts | Noto Sans, Noto Color Emoji, Noto Sans Math, Noto Sans Symbols 2 and Source Code Pro with upstream notices | Validate temporary fallback coverage and runtime packaging; proprietary Aurea Sans is deferred |
| Sounds | 16 original procedural source effects plus review preview, manifest, usage guide and license | Runtime OGA conversion, event completion, listening review and shell integration |
| Wallpapers | Nine wallpaper images | Validate dimensions, image quality, focal points and multi-display crops |
| Illustrations | Three original SVG illustrations | Add only when a real product flow requires them |
| Textures | Theme-aware CSS surface treatments | Contrast and reduced-transparency review |
| Prototypes | Two desktop HTML prototypes and one concept image | Reconcile with the desktop specification; keep simulated actions clearly identified |
| Previews | Reserved preview directories | Create useful review galleries from approved sources |

## Existing foundations

- [x] Product design principle and reference roles documented.
- [x] Desktop experience and application design specifications present.
- [x] Shared keyboard and command conventions documented.
- [x] Canonical CSS token source established.
- [x] Core component specifications: buttons, dialogs, menus, selection controls, text fields and window chrome.
- [x] Core patterns: desktop shell, application navigation, feedback/notifications and forms/settings.
- [x] Original application icon pairs and launcher icon pair retained.
- [x] Fides folder artwork v5 retained with license.
- [x] Curated Fluent UI source subset retained with upstream MIT license.
- [x] Nine original cursor SVG masters retained.
- [x] Wallpapers, illustrations, textures and original procedural sound sources retained.
- [x] Temporary font families retained with upstream notices.
- [x] Desktop prototypes retained separately from production sources.

## Consolidation and maintenance

- [x] resources/ is the canonical home for wallpapers, illustrations, textures and sounds.
- [x] Brand presentation artwork has a dedicated brand/ collection.
- [x] Third-party and original AureaOS license boundaries are documented.
- [x] design-system/tokens/index.css is the sole CSS token authority.
- [x] Human Interfaces overview and collection READMEs aligned to the current asset inventory.
- [x] Aurea Update clarified as a Settings function, not a standalone launcher application; its existing icon files are retained pending a separate asset-use decision.
- [x] Aurea Sans explicitly deferred from the current milestone.
- [ ] Validate all relative Markdown links and source references.
- [ ] Audit SPDX headers and third-party notices across the full collection.
- [ ] Review SVG source integrity and generated-output separation.

## Current priorities

### 1. Finish documentation and source hygiene

- [ ] Validate relative links and ensure inventories match the actual repository tree.
- [ ] Check duplicate, obsolete or misleading status statements.
- [ ] Confirm every retained third-party asset has its upstream license and notice.
- [ ] Keep legacy artwork untouched until its replacement or retirement is explicitly decided.

### 2. Consolidate the visual system

- [ ] Review canonical tokens for naming consistency, missing semantic roles and duplicate values.
- [ ] Ensure compatibility theme files only delegate to canonical tokens.
- [ ] Check component and pattern specifications for overlapping or contradictory rules.
- [ ] Define consistent focus, disabled, loading, error, selected and pressed states.
- [ ] Complete contrast, keyboard, scalable-text, reduced-motion and reduced-transparency guidance.
- [ ] Produce a compact light/dark reference preview of the approved design language.

### 3. Complete reusable asset integration

- [ ] Define reproducible icon export sizes and validation; keep SVG masters unchanged.
- [ ] Verify symbol mappings against exact retained source files and variants.
- [ ] Build the Xcursor output and integrate cursor states with Smithay.
- [ ] Convert approved sound sources to runtime event files and complete the event mapping.
- [ ] Validate wallpaper dimensions, quality and display crops.
- [ ] Package assets only through their owning runtime/build components.

### 4. Connect the design to the desktop

- [ ] Implement shared native components once and reuse them across first-party applications.
- [ ] Connect theme tokens and appearance changes to the native shell.
- [ ] Integrate launcher, dock, menus, dialogs, notifications and system panels with their actual services.
- [ ] Reconcile prototypes with the canonical desktop experience specification.
- [ ] Validate first-login workflows on supported hardware.
- [ ] Complete Night Light runtime integration: system-wide Vulkan output processing, capability negotiation/fallback, schedule service and persistence.
- [x] Establish the shared lock-free shell-to-compositor display policy channel through the desktop API and Wayland server state.
- [ ] Consume the display policy in the final Vulkan output stage and expose capability status to the shell.
- [ ] Add Night Light controls to Settings → Appearance and the right-side Quick Settings panel, sharing one authoritative preference state.

## Explicitly deferred

- **Aurea Sans:** original typeface design and production are not part of the current milestone. Retain the existing licensed development fonts and revisit proprietary typography later.
- New decorative illustrations, additional icon families and speculative components: add only when a concrete product requirement exists.
- Any standalone Aurea Update application: update management belongs in Settings under the current product direction.

## Runtime ownership

- **Compositor:** windows, focus, input, cursor presentation, workspaces and placement; output color policy is received through the shared desktop API, while final Vulkan processing remains pending.
- **Shell:** dock, launcher, overview, Quick Settings and notification presentation.
- **System services:** network, audio, power, display, appearance and notification state.
- **Fides and Quaero:** file-management and search workflows.
- **Packaging/build components:** installation and discovery of wallpapers, icons, fonts, cursors and sounds.

A design preview or source asset must never be reported as a working runtime feature.

## Completion criteria

A collection is ready for runtime integration only when its source, provenance, license, format, generation path, output format, accessibility behavior and runtime owner are known. Runtime integration additionally requires reproducible packaging and target-system validation.

**Destination:** one coherent, original and accessible AureaOS visual language, centrally maintained and consistently consumed by the shell and first-party applications.
