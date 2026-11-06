<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Symbol Integration

## Purpose

This document describes the planned integration direction for symbols in AureaOS applications. It does not define an implemented API or runtime loading mechanism.

The goal is to let applications use stable AureaOS catalog names without depending on upstream Fluent filenames or source directory layout.

## Symbol requests

The proposed model is for applications to request a verified AureaOS catalog name. Names shown in examples—including `folder`, `document-open`, `settings`, `network-wireless`, and `window-close`—are conceptual proposals. They have not yet been verified against or mapped to specific assets in `../source/fluent-ui/` (relative to this `catalog/` directory).

Until the catalog and integration interface are implemented, these examples must not be treated as available API names. The intended future interface should avoid requiring application code to reference source files directly.

## Variants

Variant availability must be checked against the actual source assets for each mapped symbol. `Regular` and `Filled` are illustrative variant labels, not a claim that both variants exist for every symbol.

The catalog must record a variant only after it has been verified in the source collection. Default-variant behavior remains to be defined during implementation.

## Sizes

Source dimensions and available size options must be read from verified asset metadata. This document does not establish exact dimensions, a complete size list, or an implemented logical-size API.

The future catalog may define logical sizes for application use, but each such value must be mapped to verified source assets during implementation.

## Catalog mapping

A verified mapping should preserve, when available and confirmed:

- proposed AureaOS symbol name
- exact original Fluent asset name
- verified variant
- verified source dimensions or size metadata
- source asset location

The mapping is not complete until it has been checked against the files in `../source/fluent-ui/`.

## Consistency and compatibility

The future shared interface should provide stable AureaOS names so upstream filename or organization changes do not unnecessarily affect applications. This is a design goal, not current implemented behavior.

The project intends to keep the full Fluent set available for a modern OS. Cataloging or integrating symbols must not remove or narrow the upstream source collection simply to limit the catalog.

## Future implementation

The concrete API, asset packaging format, variant defaults, logical sizing, and runtime loading mechanism remain to be defined during implementation of the AureaOS shared UI library.
