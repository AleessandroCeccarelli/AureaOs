# AureaOS Themes and Tokens

This directory contains the current CSS token source for web-based AureaOS interfaces. `tokens/index.css` imports the light and dark values. [Color System](../../docs/human-interfaces/COLOR-SYSTEM.md) documents the semantic contract those values implement. Keep this token set as the source of truth for implemented web-theme values; native toolkit mappings should use the same semantic names and values rather than define competing palettes.

Reusable visual assets, including the desktop wallpaper collection, are organized under [resources](../resources/README.md).

## Use

Import `tokens/index.css` once in a web-based AureaOS interface. The light theme is the default. Apply `data-theme="dark"` to the application root to switch the entire interface to dark mode; use `data-theme="light"` to explicitly select light mode. Remove the attribute to follow the default light theme until a system theme service is connected.

These files are design infrastructure. They are not evidence that AureaOS applications or a shared UI component library exist or consume the tokens. First-party applications should also follow the shared rules for Aurea Sans and readable fallback fonts, spacing, controls, icons, keyboard navigation, visible focus, accessibility, and plain-language terminology. The final typeface/fallback stack and the default UI language/localization policy remain open decisions. See [AureaOS Human Interfaces](../../docs/human-interfaces/README.md) for the full contract.

```html
<html data-theme="dark">
```

Use semantic variables in components, for example:

```css
.app-shell {
  color: var(--aurea-text-primary);
  background: var(--aurea-canvas);
}

.notification {
  color: var(--aurea-text-primary);
  background: var(--aurea-glass-fill-strong);
  border: 1px solid var(--aurea-glass-border);
  box-shadow: var(--aurea-glass-shadow);
  backdrop-filter: blur(28px);
}
```

Desktop services, native apps, quick settings, and notifications should map their toolkit's theme properties to the same semantic names and values. Avoid introducing app-specific copies of shared chrome colors. Ordinary app content uses opaque surface tokens; glass tokens are for transient or floating surfaces.
