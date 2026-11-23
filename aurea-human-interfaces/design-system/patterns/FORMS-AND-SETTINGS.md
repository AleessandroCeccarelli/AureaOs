<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Forms and Settings Pattern

**Status:** Design specification

## Purpose
Forms collect related information; settings change persistent preferences or system behavior. Their layout should make scope and consequences clear.

## Structure
- Group related fields or settings under clear section headings.
- Put labels before controls and supporting explanation close to the relevant setting.
- Separate unrelated groups with spacing and headings, not excessive borders.
- Use switches for immediate binary settings, radio groups for mutually exclusive choices and checkboxes for independent options.
- Show current values and units for sliders and advanced controls.

## Appearance — Night Light

Night Light is a system display preference, not an application-level color filter.

- Place the full configuration in Settings → Appearance.
- Provide an enable switch and a clearly labelled warmth/intensity slider with a visible percentage or descriptive value.
- Offer Manual, Sunset to sunrise and Custom schedule activation modes.
- Custom schedule uses local 24-hour times and validates that start and end differ.
- Sunset-to-sunrise requires an explicit, permission-aware location/timezone source. Do not silently request or assume precise location.
- Keep a custom local schedule available when location access is denied or unavailable.
- Explain that Night Light changes the color temperature of the composed display output; it is not a hardware spectral filter.
- Preserve the selected preference when the current output cannot support the required compositor processing, and explain the unavailable state rather than silently applying a partial per-window effect.

## Saving behavior
Make the save model clear. Immediate settings show progress and resulting state. Staged forms provide explicit Save and Cancel actions. Validation errors remain associated with fields. Failed operations preserve user input and report the reason where known.

## Accessibility
Use logical tab order, visible focus, field labels and accessible validation announcements. Do not rely on color alone.
