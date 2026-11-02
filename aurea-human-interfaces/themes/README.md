# AureaOS Themes and Tokens

This directory contains the current theme and token source for AureaOS interfaces.

Important: `aurea-human-interfaces/design-system/tokens/index.css` is the source of truth for the approved token set. The files in this directory are compatibility wrappers and should mirror the same semantic names and values.

The palette direction is a blue-gray foundation with a soft blue primary accent and a restrained warm accent used only for identity highlights. This keeps the system familiar and readable while preserving AureaOS personality.

## Source of truth

The authoritative token file is:

- `aurea-human-interfaces/design-system/tokens/index.css`

The theme files under this directory are kept for compatibility and should remain in sync with the token source.

## Theme behavior

- Light theme default: soft blue-gray desktop and white windows
- Dark theme: deep blue-gray desktop and dark slate windows
- Accent color: soft blue for actions and system emphasis
- Warm accent: subtle gold/amber only for identity moments, not general UI text
- Glass surfaces: translucent and readable, without sacrificing contrast

## Font status

The default system font is still DA DEFINIRE.

Aurea Sans is the intended future system font if and when it is approved and ready. Until then, this repository should not treat any specific font stack as a final system decision.

## Use

The default web theme is light mode. Apply `data-theme="dark"` to switch to dark mode.

Example:

```html
<html data-theme="dark">
```

Use semantic variables in components, not hard-coded values:

```css
.app-shell {
  color: var(--aurea-text-primary);
  background: var(--aurea-surface-desktop);
}

.notification {
  color: var(--aurea-text-primary);
  background: var(--aurea-notification-bg);
  border: 1px solid var(--aurea-notification-border);
  box-shadow: var(--aurea-notification-shadow);
  backdrop-filter: blur(24px);
}
```

## Notes

The theme files in this folder do not imply that a full desktop shell exists yet. They describe the approved design system foundation only.
