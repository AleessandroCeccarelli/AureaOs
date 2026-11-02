# Tokens

This directory contains the central source of visual values for AureaOS.

The token system is the shared vocabulary of the interface. It defines the meaning of colors, surfaces, emphasis, borders, text, shadows, and focus states.

## Goals

- one source of truth for design values
- cross-system consistency
- easy adaptation for light and dark themes
- simple mapping to UI components and desktop surfaces

## Files

- `index.css` — main token entry point
- `light.css` — light theme values
- `dark.css` — dark theme values

## Semantic model

Tokens should be descriptive, not tied to one component in isolation. For example:

- `--aurea-bg-app`
- `--aurea-bg-surface`
- `--aurea-bg-panel`
- `--aurea-accent-primary`
- `--aurea-text-primary`
- `--aurea-border-subtle`
- `--aurea-glass-fill`
- `--aurea-shadow-soft`

This reduces visual drift and creates a stable base for both the desktop shell and system applications.

## Design rules

- interface colors should be semantic, not app-specific
- used colors must remain accessible in both light and dark modes
- accent color should be strong, but not dominant across the entire system
- glass surfaces and translucency should be subtle and readable
