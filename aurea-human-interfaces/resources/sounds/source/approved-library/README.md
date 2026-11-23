<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Original Sound Library

**Version:** 0.1.0  
**Status:** Approved original source assets  
**License:** GPL-3.0-or-later — see [LICENSE.md](LICENSE.md).

This directory contains the original procedural sound effects created for
AureaOS. The sounds were synthesized specifically for this project using
original oscillator combinations, note sequences, envelopes, and light
synthesized noise. No third-party recordings, samples, or sounds extracted
from other operating systems are used.

## Audio characteristics

- **Codec/container:** Ogg Vorbis
- **File extension:** `.ogg`
- **Sample rate:** 48,000 Hz
- **Channels:** mono
- **Peak ceiling:** -3 dBFS
- **Envelope:** 8 ms fade-in and 35 ms fade-out

The exact duration and intended purpose of each effect are recorded in
[MANIFEST.json](MANIFEST.json).

## Included effects

| File | Intended purpose |
|---|---|
| `message-received.ogg` | Incoming message |
| `notification-soft.ogg` | Quiet, general notification |
| `notification-bell.ogg` | Bell-like notification option |
| `notification-gentle.ogg` | Gentle notification option |
| `notification-subtle.ogg` | Very discreet notification |
| `notification-system.ogg` | General system notice |
| `notification-ping.ogg` | Short ping notification |
| `action-confirmed.ogg` | User action confirmed |
| `action-success.ogg` | Successful operation completed |
| `warning-attention.ogg` | Warning requiring attention |
| `error-alert.ogg` | Error notification |
| `ui-feedback.ogg` | Very short interface feedback |
| `window-open.ogg` | Window opened |
| `window-close.ogg` | Window closed |
| `device-connected.ogg` | Device connected |
| `device-disconnected.ogg` | Device disconnected |

## Preview

`preview-all.ogg` is a combined listening preview of the sound palette.
It is provided for review and auditioning only; it must not be registered as
a system event sound.

## Source assets versus installed theme

These files are the preserved, original source library. They are not, by
themselves, the final installed sound theme. The freedesktop-compatible theme
lives in `../../Aurea/`; its event naming and output requirements are
documented in the parent sound-resource directory.

Use [USAGE.md](USAGE.md) for event mapping, integration rules, and the
installation destination. Do not overwrite or rename the source files when
preparing runtime assets.
