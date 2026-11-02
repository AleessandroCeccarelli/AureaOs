# AureaOS Sound Resources

This directory is the source location for the AureaOS system sound theme.

## Planned layout

- `Aurea/` — freedesktop-compatible sound theme metadata and event audio.
- `Aurea/index.theme` — theme metadata and inheritance.
- `Aurea/stereo/` — short stereo event sounds, named with standard freedesktop event identifiers where applicable.

## Design guidelines

- Keep sounds brief, restrained, and recognizable without being distracting.
- Provide notification, completion, error, device-connect/disconnect, and low-battery cues.
- Avoid sounds for routine clicks or every minor interaction.
- Keep source assets and licensing/attribution notes with the theme.
- The desktop installation target is `/usr/share/sounds/Aurea/`; this repository remains the editable source.

The event list and sound direction are documented in `SOUND-EVENTS.md`.
Original audio assets will be generated locally and added after review.