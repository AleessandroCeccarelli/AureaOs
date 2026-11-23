<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS — Planned First-Party Applications

> **Authoritative inventory:** See the [Application and Component Catalog](APPLICATION-CATALOG.md) for current names, status, programming-language policy, and technology choices. This document records concept notes for Sonus, Visio, and Explora.

## 1. Purpose and Status

This document describes three candidate applications for the AureaOS ecosystem:

- **Sonus:** music player.
- **Visio:** video player.
- **Explora:** web browser.

These applications are possible future first-party products. Their inclusion in this document does not make them Roma 1.0 commitments.

AureaOS Roma 1.0 will initially use established third-party applications for common tasks such as web browsing, email, and office productivity.

Aurea Store will provide access to compatible third-party applications through Flathub and Flatpak. These applications will run in their respective sandbox environments.

Future AureaOS releases may introduce first-party alternatives when development resources, technical priorities, and maintenance requirements make this practical.

The names, functions, and visual direction described here are product concepts, not implemented software. Application scope and technical choices remain open. Names remain subject to software-name, trademark, and domain checks.

## 2. Sonus — Music Player

**Name:** Sonus  
**Meaning:** Latin *sonus*, “sound.”

**Purpose:** A simple, integrated music player for local music collections.

### Possible functions

- Play supported local audio files.
- Provide clear playback controls and track information.
- Browse music collections, playlists, and album artwork where supported.
- Integrate with AureaOS audio controls and media controls.

The concept is a focused, approachable music player rather than an unnecessarily complex media-management suite.

The application should make common listening tasks easy without requiring users to manage a complicated media library.

The multimedia backend, codec support, and UI toolkit remain undecided.

## 3. Visio — Video Player

**Name:** Visio  
**Meaning:** Latin *visiō*, “sight” or “vision.”

**Purpose:** A focused video player for local media.

### Possible functions

- Play supported local video and audio formats using suitable multimedia infrastructure and codecs.
- Provide playback, timeline, volume, subtitle, and audio-track controls.
- Keep controls unobtrusive so the video remains the main focus.
- Use dark translucent overlays where they improve readability over video.

A more advanced media-centre experience may be considered in the future. It is not part of the current concept scope.

The choice of multimedia framework and codec support remains subject to technical, compatibility, maintenance, and licensing evaluation.

## 4. Explora — Web Browser

**Name:** Explora  
**Meaning:** Latin *explōrā*, “explore!”, the imperative of *explōrāre*.

**Purpose:** A possible future AureaOS first-party browser for everyday web navigation.

### Possible functions

- Provide tabs, an address and search bar, navigation controls, bookmarks, and browsing history.
- Prioritize readability, responsiveness, and space for web content.
- Integrate with AureaOS conventions where practical while respecting the browser engine, upstream project, security model, and licensing.

Explora is not part of the AureaOS Roma 1.0 first-release application commitment.

Roma 1.0 will rely on established third-party browsers. Aurea Store will provide access to compatible applications through Flathub and Flatpak.

If Explora is pursued in a future release, existing browser engines and upstream projects should be evaluated before deciding whether to build or customize a browser.

Security, updates, sandboxing, web compatibility, and long-term maintenance must be considered when evaluating candidates.

The browser engine and upstream project have not been selected.

## 5. Shared Visual Identity

If selected for development, Sonus, Visio, and Explora should belong to the same visual family without having identical layouts.

Their shared design rules should follow **AureaOS Human Interfaces**, the common design system and planned UI library.

The repository already contains design resources, CSS theme tokens, and documentation. The shared production-ready native UI library is not yet implemented.

### Visual principles

- **Clean, flat, geometric, and precise:** clear shapes, crisp lines, restrained borders, and deliberate spacing.
- **Material Design as a reference:** use its strengths in hierarchy, reusable components, and interaction states without copying Android's identity.
- **Shared typography:** use the system typography rules once the default font is selected. Aurea Sans remains a candidate, not an approved typeface.
- **Consistent components:** common spacing, corner radii, buttons, fields, menus, icons, focus states, animations, and accessibility rules.
- **Approved palette:** blue-gray surfaces and a soft blue primary accent form the visual foundation. Warm ivory, graphite, gold, and bronze may be used as restrained secondary accents where appropriate, not as the dominant palette.
- **Coordinated dark theme:** dark neutral and blue-gray surfaces, readable light text, and controlled accents.
- **Contextual translucency:** black translucent surfaces may be used for selected floating panels and media controls, not indiscriminately throughout every application.

The canonical source of truth for visual tokens is:

`aurea-human-interfaces/design-system/tokens/index.css`

Applications should use semantic design values rather than introducing unrelated hard-coded palettes.

### Different contexts, one design language

- **Sonus:** music collections, playlists, album artwork, and listening controls.
- **Visio:** video-first playback, with discreet controls that can fade when not needed.
- **Explora:** browser tabs, navigation, search, and web content.

The applications should share the same design principles and reusable components, while each interface remains appropriate to its purpose.

### The default Colosseum wallpaper

The default Roma wallpaper must be considered when evaluating contrast and visual balance.

Panels and floating controls must remain readable over a detailed, colourful photograph.

Coordinated wallpapers for light and dark themes may be preferable to forcing one image to work equally well in every context.

The wallpaper is a visual reference for the desktop experience, not a requirement that every application reproduce its colours or composition.

## 6. Shared System Integration

If these applications proceed, they should reuse shared AureaOS capabilities rather than duplicating common functionality.

Potential integration points include:

- Shared typography, themes, and interface components.
- System audio and media controls.
- File selection and file-access conventions.
- Notifications and desktop integration.
- Global search where relevant.

These are architectural goals, not implemented features.

AureaOS should reuse mature Linux infrastructure and existing projects where suitable, subject to compatibility, security, maintenance, and licensing review.

Each application should remain modular and maintain a clear boundary between its own functionality and shared system services.

## 7. Candidate Summary

| Application | Intended role | Current status | Roma 1.0 |
|---|---|---|---|
| Sonus | Music player | Candidate | Not committed |
| Visio | Video player | Candidate | Not committed |
| Explora | First-party web browser | Candidate | Not included |

The names were selected for their Latin roots, concise form, and fit with the AureaOS identity.

Before public release, check for existing software names, trademarks, relevant domains, and naming conflicts.

Rust remains the primary programming language for AureaOS-developed software if these applications proceed. Toolkit, backend, and browser-engine choices remain open as recorded in the [Application and Component Catalog](APPLICATION-CATALOG.md).

Their selection as candidates does not mean implementation has begun or that development is scheduled.

## 8. Relationship with Roma 1.0

The first release must prioritize a complete and usable AureaOS desktop experience and its core first-party applications.

The project will not attempt to develop a browser, email client, office suite, music player, and video player simultaneously.

Third-party applications will cover common needs while AureaOS concentrates on its own desktop environment, Aurea Wallet, Fides, Quaero, Aurea Store, and system integration.

First-party multimedia and browser applications may be evaluated for later releases based on actual user needs, available resources, and long-term maintenance costs.
