<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Human Interfaces — Status and Completion Roadmap

**Baseline:** repository `main`, 15 November 2026. This inventory distinguishes source assets, approved design decisions, missing production work and the intended destination. It is not evidence of runtime behavior or hardware validation.

## Product direction

AureaOS follows **familiarity first; refinement through integration**. Windows is the primary usability reference for desktop conventions and common workflows; KDE Plasma informs the floating dock and desktop flexibility; macOS informs consistency, visual finish and notification refinement. These are usability references, not assets or interface templates to copy.

## Canonical collections

| Area | Responsibility | Current state |
|---|---|---|
| `brand/` | Product identity and presentation | Initial artwork retained; full identity guide/logo package pending |
| `design-system/` | Tokens, components, patterns and shortcuts | Canonical token source and core specifications present; coverage/previews incomplete |
| `icons/` | Original application, system and folder artwork | Folder family and five system/app marks present; application coverage and exports pending |
| `symbols/` | Curated Fluent source and AureaOS semantic catalog | 89 concepts / 178 SVGs retained; verified catalog/API pending |
| `cursors/` | Original cursor SVG source | Nine masters present; Xcursor build and compositor integration pending |
| `fonts/` | Third-party font support and future Aurea Sans | Temporary families retained with notices; Aurea Sans not created or approved |
| `themes/` | Legacy compatibility wrappers | Must delegate to canonical tokens only |
| `resources/` | Wallpapers, illustrations, textures and sounds | Source collections present; runtime packaging and coverage incomplete |
| `components/` | Focused visual examples | Guidance present; examples to be authored |
| `prototypes/` | Visual and interaction experiments | Two HTML prototypes and one concept image retained |
| `previews/` | Review exports | Directories reserved; useful gallery pending |

## Existing source and specification work

- [x] Windows-first familiarity and KDE/macOS reference roles documented.
- [x] Desktop experience and application design specifications present.
- [x] Shared keyboard and command conventions documented.
- [x] Canonical CSS token file established at `design-system/tokens/index.css`.
- [x] Core component specifications: buttons, dialogs, menus, selection controls, text fields and window chrome.
- [x] Core patterns: desktop shell, application navigation, feedback/notifications and forms/settings.
- [x] Original Fides folder artwork v5 retained with license.
- [x] Original system artwork retained for AureaOS, Aurea AI, File Manager and Wallet.
- [x] Nine original cursor SVG masters retained.
- [x] Nine wallpaper images, three original SVG illustrations and texture CSS retained.
- [x] Original procedural sound library retained: 16 effects plus review preview, manifest, usage guide and license.
- [x] Curated Fluent UI source subset retained with upstream MIT license.
- [x] Temporary font families retained with upstream notices.
- [x] Two desktop HTML prototypes and one concept image retained.

## Structure and consistency

- [x] `resources/` established as canonical home for wallpapers, illustrations, textures and sounds.
- [x] Brand presentation artwork moved into the brand collection; redundant Human Interfaces `assets/` tree removed.
- [x] Original-project and third-party license boundaries documented.
- [x] `design-system/tokens/index.css` designated the sole CSS token authority.
- [ ] Validate all relative links after reorganization.
- [ ] Review authored files for SPDX headers and accurate status statements.

## Remaining work

### Brand and icons
- [ ] Complete canonical logo/mark masters and usage rules.
- [ ] Create missing first-party application artwork, including Quaero.
- [ ] Define icon export sizes, optical alignment, states and packaging.
- [ ] Generate previews and validate light/dark variants.

### Symbols
- [ ] Inventory exact upstream filenames and available variants.
- [ ] Verify AureaOS semantic names against actual source assets.
- [ ] Design shared symbol API, package format, fallback rules and runtime loader.
- [ ] Expand the curated source subset only for concrete product needs.

### Typography
- [ ] Define Aurea Sans brief, glyph design and language coverage.
- [ ] Validate spacing, kerning, shaping, hinting, readability and scaling.
- [ ] Establish metadata/license and build reproducible font binaries.
- [ ] Approve the production font before replacing temporary fallbacks.

### Cursors
- [ ] Define sizes and hotspots; build standard Xcursor output.
- [ ] Add reproducible generation and validation.
- [ ] Integrate cursor loading/state selection with Smithay.
- [ ] Test fractional scaling, high-DPI, reduced motion and pointer states.

### Resources and sounds
- [ ] Validate wallpaper dimensions, image quality, focal points and multi-display crops.
- [ ] Add illustrations for identified product flows only.
- [ ] Review texture contrast and reduced-transparency behavior.
- [ ] Convert approved mono `.ogg` source effects into separate runtime `.oga` stereo files where required.
- [ ] Complete event mapping; create dedicated low-battery and power cues if still required.
- [ ] Listen-test runtime copies and integrate the theme with the shell.

### Design system and previews
- [ ] Complete accessibility, typography, localization and display-scaling guidance.
- [ ] Add component contracts based on real shell/application needs.
- [ ] Produce focused examples and light/dark previews.
- [ ] Reconcile prototypes with the canonical desktop specification.
- [ ] Validate first-login workflows on supported hardware.

## Runtime ownership

- Compositor: windows, focus, input, cursor presentation, workspaces and placement.
- Shell: dock, launcher, overview, quick settings and notification presentation.
- System services: network, audio, power, display, appearance and notification state.
- Fides and Quaero: file-management and search workflows.
- Packaging: installation/discovery of wallpapers, icons, fonts, cursors and sounds.

A design preview or source asset must never be reported as a working runtime feature.

## Completion sequence

1. Finish structure, licensing, links and token compatibility.
2. Complete brand and application/system icon coverage.
3. Establish the semantic symbol catalog and API contract.
4. Develop Aurea Sans and typography rules.
5. Complete component specifications and previews.
6. Build cursor and sound runtime packages.
7. Validate desktop prototypes against specifications.
8. Integrate and test on the actual AureaOS runtime and supported hardware.

A collection is complete only when its source, provenance, license, documentation, generation path, output format, accessibility behavior and runtime owner are known. Runtime integration additionally requires reproducible builds and target-system tests.

**Destination:** a coherent, original and accessible AureaOS visual language, centrally maintained and consistently consumed by the shell and first-party applications.
