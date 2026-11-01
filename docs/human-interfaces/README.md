
# AureaOS Human Interfaces

## Purpose

AureaOS Human Interfaces is the planned shared user interface library and design system for AureaOS applications. The repository currently includes design documentation and visual resources, including CSS theme tokens. It does not yet contain an implemented shared application library or a working desktop UI.

Its purpose is to provide reusable UI components, consistent interaction patterns, and a coherent visual identity across the AureaOS desktop and applications.

## Core Principles

- **Consistency:** shared components follow common design and interaction rules.
- **Clarity:** interfaces remain understandable and easy to use.
- **Context:** each application adapts its layout to its purpose while preserving the shared design language.
- **Accessibility:** components should support accessible interaction and readable content.
- **Reusability:** common UI behavior should be implemented once and reused wherever appropriate.
- **Maintainability:** shared components should evolve without unnecessary duplication across applications.

## Scope

The shared visual and interaction contract applies to every first-party desktop and system application. App-specific layouts may differ, but applications should share:

- Semantic design tokens, light and dark themes, spacing, and typography.
- Aurea Sans as the primary typeface direction, with readable fallback fonts. The final font and fallback stack are not yet selected.
- Consistent controls, icons, focus states, and interaction patterns.
- Keyboard navigation, accessible names and feedback, scalable text, and reduced-motion support.
- Consistent product names and plain-language user-facing copy.

The shared light/dark color palette and semantic token contract are specified in [Color System](COLOR-SYSTEM.md). The current web-theme token source is [AureaOS Themes](../../aurea-human-interfaces/themes/README.md), especially `themes/tokens/index.css`. Native apps, desktop services, and notifications should map to the same semantic values rather than introduce competing design tokens.

## Relationship with AureaOS

AureaOS Human Interfaces is intended to work alongside the AureaOS System API.

Human Interfaces provides reusable visual components and interaction patterns. The System API provides shared access to system capabilities.

These are complementary responsibilities and should remain clearly separated.

## Status

Design documentation and visual resources are present. The shared library, desktop UI, and integrations described here remain planned work.

Design rules and visual resources are present, including tokens and theme files. The reusable UI component library and its application integrations remain planned. The default UI language and localization policy are open decisions.
