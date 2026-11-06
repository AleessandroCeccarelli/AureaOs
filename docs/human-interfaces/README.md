<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Human Interfaces

## Purpose

AureaOS Human Interfaces is the planned shared user interface library and design system for AureaOS applications and system surfaces.

The repository currently includes design documentation, visual resources, and token definitions. The current approved source of truth for visual values is:

- `aurea-human-interfaces/design-system/tokens/index.css`

## Core principles

- Consistency: shared components follow common design and interaction rules.
- Clarity: interfaces remain readable, understandable, and easy to use.
- Context: each application adapts to its purpose while preserving the shared system language.
- Accessibility: components support readable text, strong contrast, and accessible interaction.
- Reusability: common UI behavior should be implemented once and reused across the system.
- Maintainability: shared components should evolve without unnecessary duplication.

## Scope

The shared visual contract applies to first-party desktop and system components. App-specific layouts may differ, but applications should share:

- semantic design tokens
- light and dark themes
- spacing, layout rhythm, and structural rules
- keyboard navigation and accessible interaction patterns
- product names and plain-language user-facing copy

## Palette

The approved foundation is a blue-gray palette with a soft blue primary accent and a restrained warm accent for identity highlights only.

This replaces earlier references that treated ivory, graphite, and gold as the dominant palette basis.

## Font status

The default system font remains DA DEFINIRE.

Aurea Sans is a future candidate and may become the system font if and when it is approved. Until then, no final system font choice should be presented as definitive in docs or UI guides.

## Status

Design documentation and visual resources are present. Shared UI libraries and complete integrations remain planned work.
