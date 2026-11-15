<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Component Specifications

Canonical home for reusable interface component specifications. Each file describes one component family, its behavior, visual states, accessibility requirements and theme behavior.

## Shared visual contract

All components use the canonical semantic tokens in `../tokens/index.css`. The intended finish is Windows 11-inspired: calm blue/cyan accents, soft tonal gradients, rounded geometry, subtle borders, layered surfaces and restrained shadows. AureaOS artwork, palette and proportions remain original.

- **Surface hierarchy:** desktop canvas, opaque application content, elevated controls, floating panels and transient notification glass must remain visually distinguishable.
- **Gradients:** reserve tonal gradients for primary actions, interactive controls and selected elevated surfaces. Avoid decorative gradients on every container.
- **Transparency:** use selectively for floating system surfaces. Keep application content and dense controls readable.
- **Glass:** notification cards, dock, launcher and Quick Settings may use translucent fills, backdrop blur, saturation and a fine highlight edge. Never apply blur to text or content itself.
- **Geometry:** use shared radii, spacing and control dimensions. Do not invent per-application corner styles.
- **Depth:** combine subtle border, highlight and shadow; avoid heavy outlines and excessive glow.
- **Interaction:** hover, focus, pressed, selected, disabled, loading and error states must be recognizable in both themes.
- **Accessibility:** visible focus, adequate contrast, keyboard access, text scaling, reduced motion and reduced transparency are required.

When backdrop blur is unavailable, render a sufficiently opaque tokenized surface. Under reduced transparency, remove blur and translucent highlights while retaining surface hierarchy and readable contrast.

## Component index

- [Buttons](BUTTONS.md)
- [Text fields](TEXT-FIELDS.md)
- [Menus](MENUS.md)
- [Dialogs](DIALOGS.md)
- [Toggles and selection controls](SELECTION-CONTROLS.md)
- [Window chrome](WINDOW-CHROME.md)

## Rules

- Specifications define behavior and appearance; they do not claim a native implementation exists.
- Use semantic tokens from `../tokens/index.css`; do not introduce private palettes.
- Specify light and dark themes, keyboard focus, disabled states and reduced motion/transparency.
- Prefer one responsibility per component family and small, composable controls.
- Shared components must be reusable by first-party applications without copying implementation.
- New shared components must be added to this index.

## Component lifecycle

1. Define the user need and expected behavior.
2. Check whether an existing component already solves it.
3. Specify states, keyboard interaction, accessibility and theme behavior.
4. Add a focused visual example under `components/examples/` when useful.
5. Add light/dark previews under `previews/components/`.
6. Mark implementation status separately from design approval.
