# AureaOS Human Interfaces — Color System

## Purpose

This document defines the shared color and surface tokens for AureaOS. The desktop shell, native applications, system panels, and notifications should use these semantic tokens so that theme changes remain coherent and predictable.

The approved palette foundation is a blue-gray system with a soft blue accent and a restrained warm accent reserved for identity highlights. This keeps the interface familiar and readable while preserving a clear AureaOS personality.

## Palette

Values are sRGB hex colors. Semantic names describe their use; components should not select raw palette values independently.

### Shared accents

| Token | Value | Use |
| --- | --- | --- |
| `accent-primary` | `#6fa6ff` | Primary accent for active controls, focus, and interactive emphasis |
| `accent-strong` | `#4d8df7` | Stronger focus and active states |
| `accent-soft` | `rgba(111, 166, 255, 0.16)` | Soft fill and selection states |
| `accent-warm` | `#d8a86a` | Warm identity accent only for highlights and selected moments |
| `accent-warm-soft` | `rgba(216, 168, 106, 0.18)` | Soft warm emphasis when needed |

### Light theme

| Token | Value | Use |
| --- | --- | --- |
| `surface-desktop` | `#edf4ff` | Desktop background |
| `surface-window` | `#ffffff` | Main window surface |
| `surface-panel` | `rgba(255, 255, 255, 0.62)` | Semi-transparent panels and surfaces |
| `surface-elevated` | `#f6f9ff` | Raised sections and elevated panels |
| `text-primary` | `#121a24` | Primary text |
| `text-secondary` | `#2f3d4d` | Secondary text |
| `text-muted` | `#5d6d7f` | Tertiary and metadata text |
| `border-subtle` | `rgba(17, 24, 39, 0.08)` | Light borders and separators |
| `border-strong` | `rgba(17, 24, 39, 0.14)` | Stronger boundaries |
| `focus` | `#3d7af6` | Keyboard focus and selected state |
| `success` | `#3c7b5e` | Success state |
| `warning` | `#bc8229` | Warning state |
| `critical` | `#b6504a` | Error state |
| `dock-bg` | `rgba(247, 250, 255, 0.58)` | Dock translucent fill |
| `notification-bg` | `rgba(255, 255, 255, 0.72)` | Floating notification surface |

### Dark theme

| Token | Value | Use |
| --- | --- | --- |
| `surface-desktop` | `#0d1320` | Desktop background |
| `surface-window` | `#101b2a` | Main window surface |
| `surface-panel` | `rgba(17, 27, 42, 0.72)` | Semi-transparent panels and overlays |
| `surface-elevated` | `#152336` | Elevated surfaces |
| `text-primary` | `#edf3ff` | Primary text |
| `text-secondary` | `#d2dcf1` | Secondary text |
| `text-muted` | `#a7b5c9` | Metadata and tertiary text |
| `border-subtle` | `rgba(255, 255, 255, 0.08)` | Light separators |
| `border-strong` | `rgba(255, 255, 255, 0.12)` | Stronger boundaries |
| `focus` | `#8bb9ff` | Keyboard focus and selected state |
| `success` | `#63a77b` | Success state |
| `warning` | `#d7a750` | Warning state |
| `critical` | `#d26d63` | Error state |
| `dock-bg` | `rgba(15, 22, 31, 0.58)` | Dock translucent fill |
| `notification-bg` | `rgba(18, 27, 38, 0.72)` | Floating notification surface |

## Glass surfaces

Glass is a treatment for floating UI surfaces and notifications. It should be translucent, readable, and consistent across the shell.

| Token | Light | Dark |
| --- | --- | --- |
| `glass-fill` | `rgba(255, 255, 255, 0.28)` | `rgba(16, 27, 42, 0.36)` |
| `glass-border` | `rgba(255, 255, 255, 0.58)` | `rgba(255, 255, 255, 0.10)` |
| `glass-shadow` | `0 14px 30px rgba(17, 24, 39, 0.10)` | `0 18px 42px rgba(4, 9, 16, 0.42)` |

## Notifications

Notifications should remain compact, floating, and readable. They should follow the same semantic palette as the rest of the shell while using a more luminous glass treatment.

This keeps the system modern and premium without making notifications visually noisy or difficult to read.

## Theme behavior and accessibility

- The system theme setting selects Light or Dark for the shell, native applications, and notifications.
- Shared chrome and controls must use semantic tokens, not raw hard-coded values.
- Maintain sufficient contrast for text and important UI boundaries.
- The warm accent should be used sparingly and only for identity moments, not for general body text.
- Focus, selection, hover, and pressed states must remain distinguishable without relying only on color.

## Source of truth

The approved source of truth for visual values is:

- `aurea-human-interfaces/design-system/tokens/index.css`

The legacy theme files under `aurea-human-interfaces/themes/` remain as compatibility wrappers and should mirror the same approved values.

## Font status

The system default font is DA DEFINIRE.

Aurea Sans is the intended future candidate, subject to approval and readiness. No final font stack should be promoted as a definitive choice until the repository font database and design review approve it.
