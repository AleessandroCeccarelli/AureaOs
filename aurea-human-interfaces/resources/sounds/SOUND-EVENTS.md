<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Sound Events

Initial sound event specification for AureaOS Roma.

## Design principles

- Keep sounds short, calm, and recognizable.
- Avoid unnecessary sounds for routine interactions.
- Use consistent loudness and avoid startling peaks.
- Prefer original AureaOS sounds; use the freedesktop theme as fallback.
- Store audio files in `Aurea/stereo/` using the `.oga` format.

## Event list

| File | Event | Sound direction |
|---|---|---|
| `message-new-instant.oga` | New notification | Two soft, brief notes |
| `dialog-error.oga` | Error | Low, clear, non-aggressive tone |
| `dialog-warning.oga` | Important warning | Slightly higher tone with a short tail |
| `device-added.oga` | Device connected | Short ascending sound |
| `device-removed.oga` | Device disconnected | Short descending sound |
| `battery-low.oga` | Low battery | Two gentle pulses |
| `power-plug.oga` | Power connected | Brief, positive tone |
| `power-unplug.oga` | Power disconnected | Brief, neutral tone |
| `complete.oga` | Operation completed | Short, harmonious sequence |

## Planned audio directory

```text
Aurea/
├── index.theme
└── stereo/
    ├── message-new-instant.oga
    ├── dialog-error.oga
    ├── dialog-warning.oga
    ├── device-added.oga
    ├── device-removed.oga
    ├── battery-low.oga
    ├── power-plug.oga
    ├── power-unplug.oga
    └── complete.oga
```