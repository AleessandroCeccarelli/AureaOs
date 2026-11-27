<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Wallpaper Collection

Canonical source location for the current AureaOS desktop wallpaper collection.

Nine original AI-assisted wallpaper concepts are retained: three abstract color compositions and six Rome-inspired scenes.

- Format: JPEG.
- Intended aspect ratio: 16:9.
- Documented export target: 3840 × 2160.
- Provenance: generated specifically for AureaOS; no third-party photographs were incorporated.
- Intended use: desktop and lock-screen background candidates.

## Current default selection

**Selected default wallpaper:** `aureaos-blue-flow.jpg`

Repository path: `aurea-human-interfaces/resources/wallpapers/aureaos-blue-flow.jpg`

GitHub preview: https://github.com/AleessandroCeccarelli/AureaOs/blob/main/aurea-human-interfaces/resources/wallpapers/aureaos-blue-flow.jpg

This is the official default wallpaper for AureaOS 1.0 Roma. The blue abstract composition establishes the system's visual identity and is embedded directly into the compositor.

The other wallpapers remain available as alternatives; selecting this default does not remove or rename any artwork.

## Asset quality and future work

The artwork was generated as a contact sheet and individual panels were cropped and upscaled. Fine detail may be softer than native 4K artwork. The selected asset is intended for 3840 × 2160 output; the compositor validates the JPEG header before decoding.

Future work: distinguish source and preview files, add thumbnail metadata, dark/light suitability, focal-point guidance for overlays, multi-display aspect ratios and per-image provenance. The collection does not imply that every image is a final default selection.


## Runtime integration status — 27 November 2026

The official `aureaos-blue-flow.jpg` is embedded by the compositor, decoded through the bounded wallpaper worker and uploaded to the Vulkan texture cache. The Live ISO build stages the wallpaper source before compilation; the final-image validator checks that the compositor binary contains the selected default asset. Physical monitor presentation remains a hardware acceptance test.
