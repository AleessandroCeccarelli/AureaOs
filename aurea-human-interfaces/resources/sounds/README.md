<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Sound Theme

Contains the approved original AureaOS sound source library and the separate freedesktop-compatible runtime theme layout.

## Structure

- `source/approved-library/` — preserved procedural Ogg Vorbis source effects, preview, manifest, usage guide and license.
- `Aurea/index.theme` — freedesktop sound-theme metadata.
- `Aurea/stereo/` — destination for converted runtime event sounds.
- `SOUND-EVENTS.md` — runtime event names and source mapping.
- `AUDIO-SPECIFICATION.md` — runtime audio requirements.
- `SOUND-TESTING.md` — listening and integration checklist.

## Current state

The approved source library contains 16 original effects plus a combined review preview. Sources are Ogg Vorbis, 48 kHz, mono, with a -3 dBFS peak ceiling. They were procedurally synthesized for AureaOS; no third-party recordings or samples are included.

Preserve source files unchanged. Runtime copies require separate conversion, event naming, listening review and integration.

## Runtime destination

The intended system installation path is `/usr/share/sounds/Aurea/`. The repository's `Aurea/` directory is editable theme source, not the installed system directory.

See [source library usage](source/approved-library/USAGE.md) before preparing runtime copies.
