# Pen Tablet Profiles

Status: persistent calibration implemented; persistent monitor binding pending stable output identity plumbing.

## Persisted tablet settings

Profiles are stored at:

- `$XDG_CONFIG_HOME/aureaos/input/tablets.toml`, when XDG_CONFIG_HOME is set.
- Otherwise `$HOME/.config/aureaos/input/tablets.toml`.

Each profile is keyed by the privacy-preserving physical tablet identity produced by input discovery, not by the session-scoped `InputDeviceId` or `eventN` name.

The current profile contains:
- Normalized active area `[x, y, width, height]`.
- Rotation (0, 90, 180 or 270 degrees).
- Aspect handling (stretch or preserve).
- Physical tablet aspect ratio used by calibration.

The existing calibration constructor validates all values before they can be restored. Files use a format version, owner-only permissions on Unix, temporary-file replacement, file sync and directory sync. Malformed, invalid or newer-format files are opened read-only and are never overwritten. A failed write rolls back the in-memory profile-store mutation.

## Reconnection and reset

When a tablet is discovered with a reliable persistent identity, its saved calibration is restored into the runtime calibration map. Devices without a stable identity still work with session-only calibration; the compositor does not guess identity from a device name.

Changing calibration updates the active runtime profile and attempts to persist it. Reset removes both the runtime override and the stored profile.

## Monitor assignment: deliberate safety boundary

Tablet-to-output assignment currently remains session-scoped.

The compositor's `OutputId` is explicitly a runtime DRM topology identifier, not a persistent physical-monitor identity. Persisting that integer would risk mapping a tablet to the wrong display after reboot, GPU/connector changes or hotplug. The logical `DisplayOutput` currently does not retain the validated physical connector identity evidence discovered by DRM.

Do not serialize `OutputId` as a long-term monitor preference. The next implementation step is to carry a stable monitor identity (preferably validated EDID identity plus connector-path disambiguation) through DRM discovery into the output registry, then persist the tablet-to-monitor association against that identity. If identity is unavailable or ambiguous, keep the assignment session-only and require the user to choose again.

## Verification

The store includes source tests for round-trip/reconnection persistence, per-tablet isolation, malformed/future format protection, invalid calibration rejection and reset persistence.

No build, formatter, unit test or hardware validation has been run in this development block. Run verification together on Fedora as agreed.
