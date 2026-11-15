<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Aurea Human Interfaces

This directory defines the visual and interaction foundation for AureaOS: a familiar desktop experience, carefully integrated into one coherent product.

## Product direction

**Familiarity first; refinement through integration.**

Windows is the primary usability reference because its desktop conventions are already familiar to most PC users. AureaOS should preserve expected navigation, menu placement, file-management concepts, window controls, keyboard shortcuts and system workflows.

The experience is refined with:
- **Windows:** the primary reference for desktop information architecture, menus, launcher, file manager, window management and keyboard conventions.
- **KDE Plasma:** the floating bottom dock, adaptable desktop organization and practical customization.
- **macOS:** consistency across applications, attention to detail, coherent surfaces and polished notification behavior.

These are usability references, not assets or interfaces to copy. AureaOS combines their useful conventions into one predictable environment. Users should feel at home on first login, not be required to relearn ordinary desktop tasks.

## Core specifications

- [Application Design Guidelines](APP-DESIGN-GUIDELINES.md) — shared window structure, familiar menu conventions, application behavior and integration.
- [Desktop Experience Specification](DESKTOP-EXPERIENCE-SPEC.md) — shell, dock, launcher, workspaces, windows, notifications and quick settings.
- [Keyboard and Command Conventions](design-system/KEYBOARD-AND-COMMAND-CONVENTIONS.md) — familiar shortcuts, command placement and conflict rules.
- [Audit and Completion Roadmap](AUDIT-AND-COMPLETION-ROADMAP.md) — inventory, gaps and staged completion plan.

## Non-negotiable experience rules

- Familiar desktop tasks must work where users expect them to.
- Standard commands keep familiar names, locations and meanings.
- Keyboard shortcuts follow established Windows conventions by default.
- Window controls, title bars, menus and context menus remain predictable.
- Fides follows familiar file-manager organization and terminology.
- The floating dock and system surfaces enhance the experience without changing its basic mental model.
- Light and dark themes share the same layout, hierarchy and interaction rules.
- Visual effects never reduce readability or hide important controls.
- First-party applications share components and tokens; third-party applications remain compatible.

## Visual identity

AureaOS owns its logo, colors, icons, typography, folder artwork, wallpapers, sounds and finish. Originality belongs in the quality and consistency of these elements—not in moving familiar controls or inventing new interaction rules without a clear user benefit.

## Separation of concerns

- `design-system/` defines shared tokens, components, patterns and keyboard conventions.
- `assets/` contains wallpaper, illustrations, textures and derived visual resources.
- `brand/` contains product identity and presentation material.
- `components/` contains reusable interface examples.
- `fonts/` contains font sources and notices.
- `icons/` and `symbols/` contain graphical assets.
- `prototypes/` contains interaction and visual experiments.
- `previews/` contains visual review samples and screenshots.
- `resources/` contains approved reusable visual and audio resources.

## Design governance

Specifications describe intended behavior; they do not claim that a runtime feature is implemented. Prototypes validate visual direction and interaction flow, not hardware or compositor behavior.

A shared rule is defined once and reused. Do not create parallel palettes, shortcut systems, menu conventions or competing component libraries. Keep documents and implementation modules focused on one responsibility.
