<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Symbol Naming Convention

## Purpose

This document defines proposed naming rules for symbols that may be exposed by a future AureaOS symbol catalog.

The catalog is not yet a verified inventory or an implemented API. Names in the examples below are conceptual proposals, not confirmed matches for Fluent source assets.

## General rules

Proposed symbol names should:

- use lowercase English words
- use hyphens to separate words
- describe the symbol's meaning or function
- follow common desktop terminology where appropriate
- remain independent of original Fluent filenames
- avoid unnecessary abbreviations
- remain stable once published as part of an implemented catalog

## Semantic naming

Names should describe intended meaning or action rather than visual appearance.

Conceptual examples:

- `folder`
- `document-open`
- `document-save`
- `settings`
- `network-wireless`
- `window-close`

These examples have not yet been verified or mapped to specific assets in `../source/fluent-ui/` (relative to this `catalog/` directory).

Avoid names based only on appearance, such as:

- `blue-folder`
- `three-lines`
- `round-cross`

## Actions and objects

Use an established action-object pattern where it makes the meaning clear. The following are naming examples only, not verified asset mappings:

- `document-open`
- `document-save`
- `window-close`
- `media-play`
- `media-pause`

For common objects with an unambiguous meaning, a single object name may be suitable. These are also conceptual examples:

- `folder`
- `printer`
- `calendar`
- `camera`

## Variants

Variants should not be included in the base symbol name. A future catalog entry may record a verified variant separately from the conceptual name.

Labels such as `regular` or `filled` are examples only. Their availability must be checked against the actual source files for each symbol before being recorded.

## Source mapping

A catalog entry should preserve the exact Fluent asset name and map it to the AureaOS name only after the mapping has been verified against the source collection at `../source/fluent-ui/`.

Changing an upstream filename should not require changing a published AureaOS name, provided the mapping is maintained and verified.

## Consistency

Before adding a proposed name, check whether an existing catalog entry already represents the same meaning. Avoid synonyms unless they describe genuinely different concepts or actions.
