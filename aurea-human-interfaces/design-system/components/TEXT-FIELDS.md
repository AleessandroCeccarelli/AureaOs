<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Text Field Specification

**Status:** Design specification

## Purpose

Text fields collect or edit short values. Multi-line content uses a text area derived from the same visual language.

## Structure

A field may include a persistent visible label, supporting description, input area, optional icon/action and validation message. Placeholder text is never a replacement for a label.

## Visual treatment

Use a quiet, stable surface with shared control radius, border and typography tokens. The field should be clearly distinct from its parent surface without looking like a raised button. Focus uses the shared accent/focus treatment and a clear border or ring; avoid strong gradients inside text-entry areas. Search fields may use a compact variant but retain the same material and focus language.

## States

Define default, hover, focus-visible, filled, disabled, read-only, invalid and success states where applicable. State changes must not cause layout shifts.

## Behavior

Preserve input after validation failure. Support selection, copy, paste and keyboard navigation. Do not clear content unexpectedly. Validation feedback must be associated with the field and announced accessibly.

## Themes and accessibility

Use semantic tokens for text, placeholder, border, surface, selection and focus. Verify both themes and supported display scales. Maintain contrast in reduced-transparency mode. Placeholder contrast must remain readable and must not be the only label.
