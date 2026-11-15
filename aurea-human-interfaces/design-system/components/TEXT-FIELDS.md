<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Text Field Specification

**Status:** Design specification

## Purpose
Text fields collect or edit short values. Multi-line content uses a text area derived from the same visual language.

## Structure
A field may include a persistent visible label, supporting description, input area, optional icon/action and validation message. Placeholder text is never a replacement for a label.

## States
Define default, hover, focus-visible, filled, disabled, read-only, invalid and success states where applicable.

## Behavior
Preserve input after validation failure. Support selection, copy, paste and keyboard navigation. Do not clear content unexpectedly. Validation feedback must be associated with the field and announced accessibly.

## Search fields
Search may use a compact variant but retains the same focus, contrast and keyboard rules. Suggestions must be keyboard navigable and distinguishable from committed results.

## Themes and accessibility
Use semantic tokens for text, placeholder, border, surface, selection and focus. Verify both themes and supported display scales.
