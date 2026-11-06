<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# UI Components

This directory contains the foundational primitives for AureaOS interface components.

The goal is to keep app controls and desktop surfaces consistent across the system.

## Categories

This folder should evolve into a structured component library, for example:

- `actions/` — buttons, links, menus, commands
- `navigation/` — tabs, rails, sidebars, breadcrumbs, app launchers
- `containers/` — cards, drawers, panels, windows, forms
- `inputs/` — text fields, toggles, checkboxes, selectors
- `feedback/` — notifications, alerts, toasts, progress indicators
- `windowing/` — title bars, frames, resize logic, shell interactions

## Rules

- components must use the defined design tokens
- every component should define its states: default, hover, active, disabled, focus, selected
- avoid app-specific styling that diverges from the shared system language
- favor clarity and predictability over novelty

## Design intent

AureaOS should feel familiar to users coming from modern desktop operating systems, while still maintaining its own identity. This means:

- clear affordances
- polished motion and timing
- standard keyboard and pointer behaviors
- coherent patterns across the entire system
