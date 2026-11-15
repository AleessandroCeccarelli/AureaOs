<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Selection Controls Specification

**Status:** Design specification

## Control types

- **Checkbox:** independent option; multiple may be selected.
- **Radio group:** one choice from a mutually exclusive set.
- **Switch:** immediate on/off setting with persistent state.
- **Select/list picker:** choice from a larger set.
- **Slider:** value in a meaningful continuous or stepped range.

Do not use a switch for a one-time action or checkboxes for mutually exclusive choices.

## Shared appearance

Use the same AureaOS blue/cyan accent family, rounded geometry, subtle borders and clear focus treatment as the rest of the design system. Selected states may use a soft accent fill or gradient only where it improves recognition. Do not turn every control into a glossy button. Controls remain visually quiet until interaction or selection requires emphasis.

## Labels and feedback

Every control has a visible label. Add supporting text when consequences are not obvious. Display values and units when needed. A switch reflects actual system state; on failure, restore the real state and explain the error.

## Interaction

Support keyboard focus and activation. Radio groups support arrow navigation. Sliders support keyboard increments and announce value and range. Never show a successful state before the operation succeeds.

## States and themes

Define default, hover, focus, selected/on, off, disabled, mixed (where meaningful), pending and error states. Use semantic tokens and verify both themes. Never rely on color alone. Respect reduced motion and reduced transparency.
