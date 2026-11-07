

<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Human Interfaces

## Purpose

AureaOS Human Interfaces is the shared design system and planned user interface library for AureaOS applications and system surfaces.

Its purpose is to establish a consistent visual language, interaction model, and set of reusable interface foundations across the operating system.

The repository currently includes design documentation, visual resources, prototypes, themes, and token definitions. These resources establish the design direction but do not yet constitute a complete shared UI implementation.

The current approved source of truth for visual values is:

- `aurea-human-interfaces/design-system/tokens/index.css`

Other theme files and design resources should remain aligned with this canonical token definition.

## Core principles

- **Consistency:** shared components follow common design and interaction rules.
- **Clarity:** interfaces remain readable, understandable, and easy to use.
- **Context:** each application adapts to its purpose while preserving the shared system language.
- **Accessibility:** components support readable text, strong contrast, and accessible interaction.
- **Reusability:** common UI behavior should be implemented once and reused across the system.
- **Maintainability:** shared components should evolve without unnecessary duplication.
- **Modularity:** design foundations and reusable components should remain independent of individual applications wherever practical.
- **Predictability:** similar actions should behave consistently across system surfaces.

## Scope

The shared visual contract applies to first-party desktop and system components.

App-specific layouts may differ, but applications should share:

- Semantic design tokens.
- Light and dark themes.
- Spacing, layout rhythm, and structural rules.
- Typography rules once the system font is selected.
- Keyboard navigation and accessible interaction patterns.
- Product names and plain-language user-facing copy.
- Consistent states, feedback, and interaction behavior.

The design system should provide common foundations without forcing every application into an identical layout.

## Design system structure

The design system is intended to provide a shared foundation for:

- **Design tokens:** semantic colors, typography, spacing, borders, and other visual values.
- **Themes:** consistent light and dark appearances.
- **Components:** reusable interface elements and interaction patterns.
- **Guidelines:** accessibility, layout, content, and interaction conventions.
- **Visual resources:** icons, illustrations, wallpapers, prototypes, and other design assets.

The canonical token entry point is:

`aurea-human-interfaces/design-system/tokens/index.css`

Changes to visual values should be made consistently across the token system rather than introducing unrelated values in individual applications.

The future shared UI library should consume these foundations instead of duplicating design decisions in each component.

## Palette

The approved foundation is a blue-gray palette with a soft blue primary accent and a restrained warm accent for identity highlights only.

This replaces earlier references that treated ivory, graphite, and gold as the dominant palette basis.

The palette should remain consistent across light and dark themes while preserving appropriate contrast and readability.

Color usage should follow semantic roles rather than relying on isolated hard-coded values in individual interfaces.

## Font status

The default system font remains **to be defined**.

Aurea Sans is a future candidate and may become the system font if and when it is approved.

Until then, no final system font choice should be presented as definitive in documentation or UI guides.

Typography decisions should consider readability, accessibility, language coverage, rendering quality, and long-term availability.

## Status

Design documentation, visual resources, themes, prototypes, and token definitions are present in the repository.

The design foundations are being organized around a shared source of truth.

The following work remains planned:

- A reusable shared UI library.
- Production-ready common interface components.
- Complete integration of the design tokens into first-party applications.
- Consistent accessibility and keyboard interaction validation.
- Final system typography selection.
- Design-system documentation for implemented components and their usage.

The presence of design assets and token definitions should not be interpreted as evidence that a complete graphical desktop or shared application toolkit already exists.

