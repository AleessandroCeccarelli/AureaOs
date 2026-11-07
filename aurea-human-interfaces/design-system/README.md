<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Design System

This directory defines the visual identity of AureaOS in a simple and reusable way.

The desktop direction is intentionally familiar to people coming from Windows or macOS, while still keeping a distinct AureaOS identity. The visual language should be calm, readable, precise, and premium.

## Core design decisions

- Light theme: soft blue-gray desktop, white windows, dark readable text
- Dark theme: deep slate-blue desktop, dark windows, bright text
- Wallpaper: uncluttered blue light field with the Aurea mark softly illuminating the desktop
- Dock: full-width intelligent glass dock that floats when there is room, attaches to the bottom edge when windows need space, and can auto-hide
- Dock: Aurea mark and launcher on the left, running apps in the center, system status and controls on the right
- Notifications: floating glass-like panels with soft blur and strong readability
- Windows: clean, rounded, and light in structure
- Accent color: soft blue with a subtle warm gold highlight for identity
- Four virtual desktops, with touchpad gestures and Windows-familiar keyboard shortcuts
- Notifications appear on the left side of the desktop
- No permanent top bar in the default shell layout

## Token files

- `tokens/index.css` — shared token source
- `tokens/light.css` — light theme values
- `tokens/dark.css` — dark theme values

## Design intent

AureaOS should feel:

- familiar in workflow and behavior
- modern and premium in visual tone
- clear in readability and contrast
- calm in motion and surfaces
- unique in personality without being visually alien

## Visual rules

- windows should be mostly white or dark slate, not heavy black blocks
- the dock should be translucent and soft, not opaque and heavy
- notifications should be liquid-glass inspired, with high readability and soft blur
- gold or warm accent should be used only as a personality accent, not as a dominant color
- the shell should prioritize clarity, not clutter
