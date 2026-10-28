# AureaOS — Planned First-Party Applications

## 1. Purpose and Status

This document records three candidate applications planned for the AureaOS ecosystem: **Sonus**, **Visio**, and **Explora**.

Their names, functions, and visual direction represent the current product vision. The applications are planned concepts, not implemented software. Names remain subject to software-name, trademark, and domain checks.

## 2. Sonus — Music Player

**Name:** Sonus  
**Meaning:** Latin *sonus*, “sound.”

**Purpose:** A simple, integrated music player for local music collections.

### Planned functions

- Play supported local audio files.
- Provide clear playback controls and track information.
- Browse music collections, playlists, and album artwork where supported.
- Integrate with AureaOS audio controls and media controls.

The initial goal is a focused, approachable music player rather than an unnecessarily complex media-management suite.

## 3. Visio — Video Player

**Name:** Visio  
**Meaning:** Latin *visiō*, “sight” or “vision.”

**Purpose:** A focused video player for local media.

### Planned functions

- Play supported local video and audio formats using suitable multimedia infrastructure and codecs.
- Provide playback, timeline, volume, subtitle, and audio-track controls.
- Keep controls unobtrusive so the video remains the main focus.
- Use dark translucent overlays where they improve readability over video.

A more advanced media-centre experience may be considered in the future. It is not part of the initial scope.

## 4. Explora — Web Browser

**Name:** Explora  
**Meaning:** Latin *explōrā*, “explore!”, the imperative of *explōrāre*.

**Purpose:** A browser for everyday web navigation.

### Planned functions

- Provide tabs, an address and search bar, navigation controls, bookmarks, and browsing history.
- Prioritize readability, responsiveness, and space for web content.
- Integrate with AureaOS conventions where practical while respecting the browser engine, upstream project, security model, and licensing.

The browser engine and upstream project have not yet been selected. Existing browser technology should be evaluated before deciding whether to build or customize a browser.

## 5. Shared Visual Identity

Sonus, Visio, and Explora should belong to the same visual family without having identical layouts.

Their shared design rules will be defined by **AureaOS Human Interfaces**, the planned common interface library and design system.

### Visual principles

- **Clean, flat, geometric, and precise:** clear shapes, crisp lines, restrained borders, and deliberate spacing.
- **Material Design as a reference:** use its strengths in hierarchy, reusable components, and interaction states without copying Android’s identity.
- **Shared typography:** Aurea Sans is the intended system typeface, with consistent text hierarchy.
- **Consistent components:** common spacing, corner radii, buttons, fields, menus, icons, focus states, animations, and accessibility rules.
- **Light surfaces first:** warm white, ivory, and pale grey, with gold and bronze used as restrained accents rather than dominant colours.
- **Coordinated dark theme:** dark neutral surfaces, readable light text, and controlled accents.
- **Contextual translucency:** black translucent surfaces may be used for selected floating panels and media controls, not indiscriminately throughout every application.

### Different contexts, one design language

- **Sonus:** music collections, playlists, album artwork, and listening controls.
- **Visio:** video-first playback, with discreet controls that can fade when not needed.
- **Explora:** browser tabs, navigation, search, and web content.

The applications should share the same design principles and reusable components, while each interface remains appropriate to its purpose.

### The default Colosseum wallpaper

The default Roma wallpaper must be considered when evaluating contrast and visual balance. Panels and floating controls must remain readable over a detailed, colourful photograph.

Coordinated wallpapers for light and dark themes may be preferable to forcing one image to work equally well in every context.

## 6. Shared System Integration

Where suitable, the applications should reuse shared AureaOS capabilities rather than duplicating common functionality.

Potential integration points include:

- Shared typography, themes, and interface components.
- System audio and media controls.
- File selection and file-access conventions.
- Notifications and desktop integration.
- Global search where relevant.

These are architectural goals, not implemented features. AureaOS should reuse mature Linux infrastructure and existing projects where suitable, subject to compatibility, security, maintenance, and licensing review.

## 7. Candidate Summary

| Application | Intended role | Status |
|---|---|---|
| Sonus | Music player | Candidate |
| Visio | Video player | Candidate |
| Explora | Web browser | Candidate |

The names were selected for their Latin roots, concise form, and fit with the AureaOS identity. Before public release, check for existing software names, trademarks, relevant domains, and naming conflicts.

The applications remain planned concepts. Their selection does not mean implementation has begun or that their final technical choices have been made. 
