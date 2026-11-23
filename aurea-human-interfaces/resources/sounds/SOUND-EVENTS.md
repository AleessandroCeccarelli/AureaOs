<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Sound Events

Defines intended freedesktop-compatible runtime event names and maps them to the approved AureaOS original sound source library.

## Design principles

- Keep sounds brief, calm and recognizable.
- Avoid unnecessary sounds for routine interactions.
- Keep perceived loudness consistent and avoid startling peaks.
- Prefer original AureaOS effects; inherit freedesktop sounds as fallback.
- Preserve approved source files unchanged.
- Runtime copies use `.oga` and must be genuinely converted to the required audio profile.

## Current source-to-event mapping

| Runtime event file | Event | Approved source | Status |
|---|---|---|---|
| `message-new-instant.oga` | New notification | `message-received.ogg` | Proposed mapping; runtime copy pending |
| `dialog-error.oga` | Error | `error-alert.ogg` | Proposed mapping; runtime copy pending |
| `dialog-warning.oga` | Important warning | `warning-attention.ogg` | Proposed mapping; runtime copy pending |
| `device-added.oga` | Device connected | `device-connected.ogg` | Proposed mapping; runtime copy pending |
| `device-removed.oga` | Device disconnected | `device-disconnected.ogg` | Proposed mapping; runtime copy pending |
| `complete.oga` | Operation completed | `action-success.ogg` | Proposed mapping; runtime copy pending |
| `battery-low.oga` | Low battery | No dedicated source | Not available; create and review dedicated cue |
| `power-plug.oga` | Power connected | No dedicated source | Not available; create and review dedicated cue |
| `power-unplug.oga` | Power disconnected | No dedicated source | Not available; create and review dedicated cue |

Other source effects may support differentiated notification preferences or optional UI feedback. They are not automatically assigned to system events.

## Runtime layout

```text
Aurea/
├── index.theme
└── stereo/
    ├── message-new-instant.oga
    ├── dialog-error.oga
    ├── dialog-warning.oga
    ├── device-added.oga
    ├── device-removed.oga
    ├── complete.oga
    ├── battery-low.oga
    ├── power-plug.oga
    └── power-unplug.oga
```

Do not create placeholder audio files. An event remains unavailable until its dedicated runtime asset has been generated, reviewed and tested.
