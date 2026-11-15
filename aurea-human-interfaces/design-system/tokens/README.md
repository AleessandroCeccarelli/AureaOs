<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Canonical Design Tokens

`index.css` is the single source of truth for AureaOS CSS design tokens.

It defines the default light theme, the dark theme override selected by `:root[data-theme='dark']`, and shared accessibility overrides such as `prefers-reduced-transparency`. Compatibility files `light.css` and `dark.css` import this file and must not duplicate values.

Current token families cover desktop/window surfaces, panels, text, borders, accent/status colors, focus, dock, notifications, glass materials, shadows, radii, spacing and temporary font-family aliases.

Propose missing shared tokens here before introducing component-local substitutes. Components consume semantic tokens, not private palettes. Preserve the same hierarchy and interaction model in both themes. Transparency and shadow never override readable contrast. Reduced-transparency behavior belongs in this canonical file so consumers do not need to import legacy theme wrappers.

Runtime appearance state belongs to the system appearance service. CSS tokens and media-query fallbacks do not by themselves implement a native theme service.
