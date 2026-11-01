# AureaOS Human Interfaces — Color System

## Purpose

This document defines the shared color and surface tokens for AureaOS. The desktop shell, native applications, system panels, and notifications must use these semantic tokens so that a theme change updates the whole experience together.

The visual direction pairs the clarity and layered desktop surfaces of a modern Windows-style interface with AureaOS's warm ivory, graphite, and gold identity. Notifications use compact, luminous glass cards with layered highlights and soft depth, while keeping AureaOS typography, spacing, and gold accent. These are design cues, not copies of another system's interface.

## Palette

Values are sRGB hex colors. Semantic names describe their use; components should not select palette values independently.

### Brand and shared accents

| Token | Value | Use |
| --- | --- | --- |
| `accent` | `#A97818` | Primary Aurea gold; links, selected controls, key actions |
| `accent-hover` | `#8E6110` | Hover and stronger emphasis in light theme |
| `accent-soft` | `#F4E6C7` | Soft gold fills and selection backgrounds in light theme |
| `accent-dark` | `#E6BD68` | Gold accent in dark theme |
| `accent-dark-hover` | `#F0CE86` | Hover and stronger emphasis in dark theme |
| `accent-dark-soft` | `#49391F` | Soft gold fills and selection backgrounds in dark theme |

### Light theme

| Token | Value | Use |
| --- | --- | --- |
| `canvas` | `#F3F4F7` | Desktop and app canvas |
| `surface` | `#FFFFFF` | Main app and window surface |
| `surface-secondary` | `#F8F8FA` | Secondary sections, cards, input fields |
| `surface-raised` | `#FFFFFF` | Menus, dialogs, popovers |
| `surface-sunken` | `#E9EBF0` | Recessed controls and selected neutral tracks |
| `text-primary` | `#20232A` | Main text and icons |
| `text-secondary` | `#5E6470` | Supporting text |
| `text-tertiary` | `#747B87` | Metadata and low emphasis text |
| `text-disabled` | `#969BA5` | Disabled text and icons |
| `border` | `#D9DCE3` | Standard borders and separators |
| `border-strong` | `#B9BEC9` | Emphasized boundaries and focus context |
| `focus-ring` | `#A97818` | Keyboard focus outline |
| `selection` | `#E9D8B3` | Selected item background |
| `hover` | `#ECEEF2` | Neutral hover background |
| `pressed` | `#DFE2E8` | Neutral pressed background |

### Dark theme

| Token | Value | Use |
| --- | --- | --- |
| `canvas` | `#17191E` | Desktop and app canvas |
| `surface` | `#202329` | Main app and window surface |
| `surface-secondary` | `#272A31` | Secondary sections, cards, input fields |
| `surface-raised` | `#30343D` | Menus, dialogs, popovers |
| `surface-sunken` | `#191B20` | Recessed controls and selected neutral tracks |
| `text-primary` | `#F4F2ED` | Main text and icons |
| `text-secondary` | `#C2C4CA` | Supporting text |
| `text-tertiary` | `#A0A4AE` | Metadata and low emphasis text |
| `text-disabled` | `#777C87` | Disabled text and icons |
| `border` | `#3B3F49` | Standard borders and separators |
| `border-strong` | `#555B68` | Emphasized boundaries and focus context |
| `focus-ring` | `#E6BD68` | Keyboard focus outline |
| `selection` | `#49391F` | Selected item background |
| `hover` | `#333741` | Neutral hover background |
| `pressed` | `#3C414C` | Neutral pressed background |

### Semantic status colors

Status colors are shared in both themes, with foreground and container pairs to maintain readable contrast. Do not use status color as the only indication of state; pair it with an icon or text label.

| Role | Light foreground / container | Dark foreground / container |
| --- | --- | --- |
| Success | `#237A52` / `#E3F3E9` | `#75D19C` / `#203B2D` |
| Warning | `#8A5A00` / `#FFF1D2` | `#F0C56B` / `#44351D` |
| Error | `#B33A3A` / `#FBE9E7` | `#F28B82` / `#482725` |
| Information | `#2866A8` / `#E7F0FB` | `#8AB4F8` / `#20334D` |

## Glass surfaces

Glass is a surface treatment applied to shared semantic colors, not a separate component palette. Use it for the floating desktop bar, launcher, quick settings, calendar, contextual overlays, and notifications. Ordinary reading and editing surfaces remain opaque by default.

| Token | Light | Dark |
| --- | --- | --- |
| `glass-fill` | `rgba(250, 250, 252, 0.76)` | `rgba(34, 37, 44, 0.78)` |
| `glass-fill-strong` | `rgba(250, 250, 252, 0.90)` | `rgba(34, 37, 44, 0.92)` |
| `glass-highlight` | `rgba(255, 255, 255, 0.72)` | `rgba(255, 255, 255, 0.10)` |
| `glass-border` | `rgba(255, 255, 255, 0.72)` | `rgba(255, 255, 255, 0.12)` |
| `glass-shadow` | `0 12px 36px rgba(28, 35, 49, 0.16)` | `0 16px 42px rgba(0, 0, 0, 0.38)` |

Apply a backdrop blur in the 20–32 px range for large panels and 24–40 px for compact notification cards. Treat these as starting values and provide an opaque fallback when backdrop filtering is unavailable or reduced transparency is enabled. Keep a thin border and top-edge highlight so the panel stays legible over both bright and dark wallpaper.

## Notifications

Notifications share the global theme and semantic text/accent tokens. Their glass treatment may be more luminous and compact than ordinary panels: use `glass-fill-strong`, a soft highlight, a fine border, and a diffuse shadow. Use rounded corners consistently with the shared component radius scale. Keep source, title, body, timestamp, and actions distinct through typography and spacing rather than separate colors. Status colors may indicate urgency or outcome, but the base notification remains neutral glass.

In dark mode, preserve the charcoal glass fill and use light text; do not turn notifications into pure black cards. In light mode, use a warm near-white fill with dark text. When transparency is disabled, use `surface-raised` and retain the border and shadow.

## Theme behavior and accessibility

- The system theme setting selects Light or Dark for the shell, native applications, and notifications. Apps may expose content-specific colors, but shared chrome and controls follow these tokens.
- Components use semantic names such as `surface-raised` and `text-primary`; they must not hard-code light or dark hex values.
- Maintain at least 4.5:1 contrast for normal text and 3:1 for large text and meaningful UI boundaries. Recheck contrast for translucent surfaces against representative light and dark wallpapers.
- Gold is an identity accent, not a general text color. Use `text-primary` for body copy and verify contrast when gold is used for small labels or icons.
- Focus, selection, disabled, hover, and pressed states must remain distinguishable without relying only on color.
- Respect reduced-transparency and reduced-motion preferences. In reduced-transparency mode, replace glass fills with the corresponding opaque semantic surface.

## CSS reference

The following demonstrates the semantic contract for web-based native surfaces. Other toolkits should expose equivalent names and values through their native theme mechanism.

```css
:root,
[data-theme="light"] {
  color-scheme: light;
  --aurea-canvas: #f3f4f7;
  --aurea-surface: #ffffff;
  --aurea-surface-secondary: #f8f8fa;
  --aurea-surface-raised: #ffffff;
  --aurea-text-primary: #20232a;
  --aurea-text-secondary: #5e6470;
  --aurea-border: #d9dce3;
  --aurea-accent: #a97818;
  --aurea-focus-ring: #a97818;
  --aurea-glass-fill: rgba(250, 250, 252, 0.76);
  --aurea-glass-fill-strong: rgba(250, 250, 252, 0.9);
  --aurea-glass-highlight: rgba(255, 255, 255, 0.72);
  --aurea-glass-border: rgba(255, 255, 255, 0.72);
  --aurea-glass-shadow: 0 12px 36px rgba(28, 35, 49, 0.16);
}

[data-theme="dark"] {
  color-scheme: dark;
  --aurea-canvas: #17191e;
  --aurea-surface: #202329;
  --aurea-surface-secondary: #272a31;
  --aurea-surface-raised: #30343d;
  --aurea-text-primary: #f4f2ed;
  --aurea-text-secondary: #c2c4ca;
  --aurea-border: #3b3f49;
  --aurea-accent: #e6bd68;
  --aurea-focus-ring: #e6bd68;
  --aurea-glass-fill: rgba(34, 37, 44, 0.78);
  --aurea-glass-fill-strong: rgba(34, 37, 44, 0.92);
  --aurea-glass-highlight: rgba(255, 255, 255, 0.1);
  --aurea-glass-border: rgba(255, 255, 255, 0.12);
  --aurea-glass-shadow: 0 16px 42px rgba(0, 0, 0, 0.38);
}

@media (prefers-reduced-transparency: reduce) {
  :root,
  [data-theme] {
    --aurea-glass-fill: var(--aurea-surface);
    --aurea-glass-fill-strong: var(--aurea-surface-raised);
  }
}
```
