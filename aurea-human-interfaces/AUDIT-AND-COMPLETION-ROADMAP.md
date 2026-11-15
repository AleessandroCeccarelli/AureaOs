<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Human Interfaces — Audit and Completion Roadmap

**Audit baseline:** GitHub `main`, 15 November 2026  
**Purpose:** Record the current contents of this directory, identify gaps and define a controlled plan for completing the AureaOS desktop design foundation.

This is a repository-content audit, not a claim that visual assets, prototypes or runtime features have been tested on hardware.

## 1. Current inventory

The directory already contains a substantial foundation:

| Area | Existing content | Assessment |
|---|---|---|
| `assets/` | Folder guides for wallpapers, backgrounds, illustrations and textures | Structure exists; most actual reusable content currently lives under `resources/` |
| `brand/` | Brand README | Documentation exists; brand usage rules and a clean canonical logo package are still needed |
| `components/` | Empty examples and specifications folders | Not yet a usable component catalogue |
| `design-system/` | Overview, token files, component and pattern guides | Good starting point; component and pattern specifications are mostly missing |
| `fonts/` | Noto Sans, Noto Emoji, Noto Symbols, Noto Math and Source Code Pro sources with notices | Useful development coverage; Aurea Sans is not yet an approved production font |
| `icons/` | Original AureaOS marks and application/system SVGs; generated and application folders reserved | Initial identity assets exist; catalog, export rules and application icon coverage need work |
| `previews/` | Reserved preview folders | No meaningful review gallery yet |
| `prototypes/` | Two interactive HTML desktop concepts and a concept image | Most important visual reference; prototypes are not production UI and simulate system actions |
| `resources/` | Wallpapers, illustrations, texture CSS and sound documentation | Useful reusable resources; sound specification exists but audio files are not present |
| `symbols/` | 178 Fluent UI SVG files: 89 Regular and 89 Filled, plus catalog guidance | Curated third-party reference subset, not a complete AureaOS symbol library or runtime API |
| `themes/` | Light and dark compatibility CSS files | Exists, but must remain synchronized with canonical design tokens |

## 2. Important findings

### 2.1 The desktop is represented primarily by prototypes

The two desktop prototypes are:

- `prototypes/2026-11-aurea-desktop-vision-01.html`
- `prototypes/2026-11-aurea-desktop-preview-02.html`

They are valuable design references. Their controls and hardware/system actions are simulations. They must not be treated as proof that the native shell already implements those behaviors.

The next step is to turn the chosen desktop direction into explicit specifications for dock, launcher, windows, workspaces, notifications and quick settings.

### 2.2 Shared component library is the largest structural gap

The following areas are currently only placeholders or introductory documentation:

- `components/examples/`
- `components/specifications/`
- `design-system/components/`
- `design-system/patterns/`
- `previews/components/`

There is no complete, versioned catalogue of the shared visual components yet. This is a priority because first-party applications need to reuse the same controls and window patterns.

### 2.3 Theme source needs a strict ownership model

The canonical token source is:

`design-system/tokens/index.css`

The files under `themes/` are documented as compatibility wrappers. The separate `light.css` and `dark.css` files under `design-system/tokens/` are theme-specific token values.

These files must be checked for semantic parity and a clear import contract. Applications must not choose between multiple competing sources of truth.

### 2.4 Typography is not finalized

Noto and Source Code Pro are retained as development/coverage resources. Aurea Sans is a future project and is not approved as a production font yet.

Still needed:
- Font family and role specification.
- Type scale and line-height rules.
- Language and glyph coverage validation.
- Font licensing and metadata review.
- Actual Aurea Sans production files only after approval.

Do not label an unapproved font as the final system font.

### 2.5 Icon and symbol governance is incomplete

Original AureaOS system and brand artwork already exists. A curated Fluent UI reference subset is also present.

Still needed:
- An inventory of original AureaOS icons.
- Rules for application icons, system icons and status symbols.
- Verified mappings between stable AureaOS symbol names and exact source assets.
- Clear distinction between original AureaOS artwork and third-party reference assets.
- Export sizes, optical alignment and light/dark behavior.
- A shared symbol API only when the runtime design is ready.

The current symbol catalog correctly warns that its proposed names and mappings are not yet a verified or implemented API.

### 2.6 Resource locations need clearer ownership

There are both `assets/` and `resources/` trees describing wallpapers, illustrations and textures. The current resources README identifies `resources/wallpapers/` as the canonical wallpaper location.

Keep one canonical copy of each asset. The `assets/` tree should either hold distinct design-source material or clearly point to the canonical reusable resources; do not duplicate files simply to make both trees look populated.

### 2.7 Audio is specified but not produced

Sound event names, target durations and audio review checklists exist. The planned `.oga` files are not present in the current tree.

This is an unfinished asset-production task, not a reason to create empty or fake audio files.

## 3. Missing design specifications

The following documents should be completed before implementing a large number of UI components:

1. Desktop experience — dock, launcher, workspaces, notifications and quick settings.
2. Window management — title bar, resize, maximize, snap, tiling, focus and overview.
3. Navigation and application menu — canonical menu names, ordering, keyboard behavior and contextual menus.
4. Component interaction states — hover, focus, pressed, selected, disabled, loading and error.
5. Typography — hierarchy, sizing, line height, fallback and localization.
6. Iconography — original icons, symbol mapping, sizes, stroke/fill and state rules.
7. Accessibility — keyboard, scaling, contrast, reduced motion, reduced transparency and assistive technology.
8. Responsive behavior — minimum sizes, display scaling, small screens and multi-monitor considerations.
9. Notifications and system panels — grouping, priority, placement, dismissal and focus behavior.
10. Third-party integration — supported theme/toolkit integration levels and limits.

The application design guidelines and desktop experience specification are the initial documents for this work.

## 4. Recommended folder responsibilities

Keep the existing top-level structure and give each area one clear responsibility:

- `assets/`: source artwork and design inputs that are not runtime-ready resources.
- `brand/`: AureaOS identity, logo usage, clear-space and brand rules.
- `components/`: practical component examples and component specification index.
- `design-system/`: semantic tokens, reusable components and interaction patterns.
- `fonts/`: source fonts, licenses, generated/approved fonts and typography documentation.
- `icons/`: original AureaOS application, system and brand icons, plus generated exports.
- `previews/`: reviewed visual samples for components, fonts, icons and themes.
- `prototypes/`: interactive and static design experiments.
- `resources/`: reusable runtime-ready wallpapers, illustrations, textures and sounds.
- `symbols/`: curated third-party symbol sources, licensing and verified AureaOS catalog mapping.
- `themes/`: compatibility entry points only; canonical tokens remain in the design system.

Avoid creating parallel folders or duplicate assets.

## 5. Completion roadmap

### Phase A — Define the desktop contract

- [x] Document application visual and behavior guidelines.
- [x] Document the desktop experience at shell level.
- [ ] Specify the dock in detail.
- [ ] Specify launcher layout, search behavior and dismissal.
- [ ] Specify window controls, snap layouts and overview.
- [ ] Specify quick settings and notification behavior.
- [ ] Map prototype controls to intended native shell responsibilities.

### Phase B — Complete the design system

- [ ] Define semantic token ownership and CSS import contract.
- [ ] Add typography scale and layout/spacing guidance.
- [ ] Specify common controls: button, text field, toggle, checkbox, selector and slider.
- [ ] Specify menus, context menus, tooltips and popovers.
- [ ] Specify dialogs, sheets, notifications and progress feedback.
- [ ] Add component state and accessibility requirements to each specification.

### Phase C — Establish visual asset governance

- [ ] Publish brand and logo usage rules.
- [ ] Inventory original AureaOS icons and application icon requirements.
- [ ] Verify symbol catalog entries against exact source files.
- [ ] Define icon export sizes and optical alignment.
- [ ] Define wallpaper variants, aspect ratios and attribution.
- [ ] Complete typography approval and font packaging.
- [ ] Produce and review original system sounds.

### Phase D — Make the library reviewable

- [ ] Add a component index and focused examples.
- [ ] Add light/dark component previews.
- [ ] Add font specimen previews.
- [ ] Add icon and symbol catalog previews.
- [ ] Add theme comparison previews.
- [ ] Keep screenshots and previews clearly labeled as design references, not runtime evidence.

### Phase E — Connect design to implementation

- [ ] Map each approved component to its intended Rust/native implementation owner.
- [ ] Keep design specifications separate from runtime code.
- [ ] Confirm that system controls represent real service capabilities.
- [ ] Validate the native shell against the approved prototype and specifications.
- [ ] Record implementation status separately from design approval.

## 6. Working rules

- One file, one responsibility; keep documents focused and avoid monolithic specifications.
- Do not create placeholder assets that imply a feature is complete.
- Do not duplicate canonical wallpaper, icon, font or theme files.
- Do not silently change approved visual decisions in individual applications.
- Do not claim prototype interactions are implemented system functionality.
- Preserve third-party license notices and source attribution.
- Review both light and dark appearances for every shared component.
- Keep accessibility and reduced-motion behavior in the initial specification, not as a later patch.

## 7. Audit conclusion

AureaOS Human Interfaces already has a meaningful foundation: desktop prototypes, semantic color tokens, theme files, original identity artwork, fonts, wallpapers, illustrations, sounds documentation and a curated symbol reference set.

The biggest missing pieces are not more decorative assets. They are the detailed desktop behavior specifications, the reusable component catalogue, typography and icon governance, accessibility rules, and a reviewable set of component previews.

The recommended order is to define the desktop contract first, then build the shared component library around it. This avoids designing the same dock, menu, window header or control differently in separate parts of AureaOS.
