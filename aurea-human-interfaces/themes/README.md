# AureaOS Themes and Tokens

The shared CSS custom properties in this directory implement the palette defined in [Color System](../../docs/human-interfaces/COLOR-SYSTEM.md).

Reusable visual assets, including the desktop wallpaper collection, are organized under [resources](../resources/README.md).

## Use

Import `tokens/index.css` once in a web-based AureaOS interface. The light theme is the default. Apply `data-theme="dark"` to the application root to switch the entire interface to dark mode; use `data-theme="light"` to explicitly select light mode. Remove the attribute to follow the default light theme until the system theme service is connected.

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
