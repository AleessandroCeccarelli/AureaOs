<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Sound Theme — Testing Checklist

Use this checklist for converted runtime files and the installed Aurea sound theme. The original source library is reviewed separately and must remain unchanged.

## Source and mapping

- [ ] Source file exists in `source/approved-library/`.
- [ ] Source purpose matches the event mapping in `SOUND-EVENTS.md`.
- [ ] No unrelated source effect has been assigned to an event.
- [ ] Dedicated battery/power cues are created only if those events are required.

## Runtime file and format

- [ ] Runtime event file uses the exact `.oga` name in `SOUND-EVENTS.md`.
- [ ] Ogg Vorbis encoding is verified; changing the extension alone is not conversion.
- [ ] Sample rate, stereo channels and peak level meet `AUDIO-SPECIFICATION.md`.
- [ ] File plays correctly and is not corrupted.
- [ ] Duration and silence are appropriate.

## Audio quality

- [ ] Perceived loudness is consistent.
- [ ] No clipping, distortion, clicks or unintended noise.
- [ ] No sudden or startling peaks.
- [ ] Sounds remain recognizable at low playback volume.
- [ ] Repeated playback is not irritating or distracting.

## Integration

- [ ] Theme metadata installs correctly.
- [ ] Missing events fall back to the freedesktop theme.
- [ ] Playback works through the target desktop notification/audio path.
- [ ] No duplicate cues play for the same event.
- [ ] Reduced-volume and muted behavior are respected.

## Approval record

- Review date:
- Reviewer:
- Source file:
- Runtime event:
- Result: Pending / Approved / Needs revision
- Notes:
