<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Using the AureaOS Original Sound Library

This document explains which effects are available, where they belong, and
how to integrate them without confusing the preserved source library with
the installed desktop sound theme.

## 1. Repository locations

Keep the original, approved source files here:

```text
aurea-human-interfaces/resources/sounds/source/approved-library/
```

The freedesktop-compatible theme metadata and runtime event sounds belong
here:

```text
aurea-human-interfaces/resources/sounds/Aurea/
├── index.theme
└── stereo/
```

The intended system installation destination is:

```text
/usr/share/sounds/Aurea/
```

The repository is the editable source; the installation directory is the
system-wide runtime destination. Do not place the review preview or the
source-library documentation in the runtime theme directory.

## 2. Available sound roles

Use the following mapping as the starting point when connecting desktop
events to the original sound palette:

| Original source | Intended event / use |
|---|---|
| `message-received.ogg` | Incoming message |
| `notification-soft.ogg` | Default quiet notification |
| `notification-gentle.ogg` | Alternative gentle notification |
| `notification-subtle.ogg` | Low-priority notification |
| `notification-bell.ogg` | More noticeable notification, when explicitly appropriate |
| `notification-ping.ogg` | Short notification cue |
| `notification-system.ogg` | General system notification |
| `action-confirmed.ogg` | Explicit confirmation of a user action |
| `action-success.ogg` | Successful completion |
| `warning-attention.ogg` | Warning that needs the user's attention |
| `error-alert.ogg` | Error or failed operation |
| `ui-feedback.ogg` | Optional, very brief interface feedback |
| `window-open.ogg` | Optional window-open event |
| `window-close.ogg` | Optional window-close event |
| `device-connected.ogg` | Device connected |
| `device-disconnected.ogg` | Device disconnected |

Notification variants are alternatives, not a sequence. Select one default
notification sound and use the others only for clearly differentiated
notification classes or user-selectable preferences. Do not play several
notification variants for the same event.

## 3. Mapping to freedesktop event names

The existing AureaOS sound-event specification defines these runtime event
names:

| AureaOS event file | Suggested source |
|---|---|
| `message-new-instant.oga` | `message-received.ogg` |
| `dialog-error.oga` | `error-alert.ogg` |
| `dialog-warning.oga` | `warning-attention.ogg` |
| `device-added.oga` | `device-connected.ogg` |
| `device-removed.oga` | `device-disconnected.ogg` |
| `complete.oga` | `action-success.ogg` |

This is a proposed integration mapping, not a claim that the runtime files
have already been generated. The current source library does not contain
dedicated low-battery, power-connected, or power-disconnected effects.
Do not reuse an unrelated sound for those events; create and review dedicated
cues if those events are required.

## 4. Format and conversion requirements

The preserved source files are Ogg Vorbis, 48 kHz, mono, with the `.ogg`
extension. The current AureaOS audio specification describes runtime theme
assets as Ogg Vorbis in an `.oga` container, 48 kHz, stereo, with a target
peak no higher than -3 dBFS.

Before placing a source effect in `Aurea/stereo/`:

1. Keep the original `.ogg` unchanged in `source/approved-library/`.
2. Convert or export a separate runtime copy to the format required by
   `AUDIO-SPECIFICATION.md`.
3. Use the event filename listed in `SOUND-EVENTS.md`.
4. Check duration, perceived loudness, channel output, clipping, and audible
   clicks after conversion.
5. Record the result in `SOUND-TESTING.md`.

Changing the extension alone is not a format conversion. Do not mark an event
as integrated until the generated runtime file has been listened to and
tested.

## 5. Playback guidelines

- Keep routine interactions silent unless sound provides meaningful feedback.
- Reserve warning and error cues for events that genuinely need attention.
- Avoid overlapping multiple cues for one event.
- Keep window sounds and tiny UI feedback optional; provide a way to disable
  them if they become distracting.
- Respect the user's system volume, mute state, and sound preferences.
- Do not use `preview-all.ogg` as a notification or desktop event.

## 6. Licensing and provenance

The original source effects are licensed under GPL-3.0-or-later. See
[LICENSE.md](LICENSE.md) and the repository-root `LICENSE` for the full
license text. The library contains original procedural synthesis and does
not include third-party recordings or samples.
