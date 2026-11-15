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

## Saving behavior
Make the save model clear. Immediate settings show progress and resulting state. Staged forms provide explicit Save and Cancel actions. Validation errors remain associated with fields. Failed operations preserve user input and report the reason where known.

## Accessibility
Use logical tab order, visible focus, field labels and accessible validation announcements. Do not rely on color alone.
