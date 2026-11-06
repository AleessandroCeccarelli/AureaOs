<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Symbols

## Purpose

This directory holds the symbol source assets and the documentation for a planned AureaOS symbol catalog.

AureaOS intends to keep the full Fluent UI System Icons set available as a source collection, so the operating system and its applications can draw on the coverage needed by a modern OS. The AureaOS catalog is intended to provide stable, descriptive names and mappings over time; that catalog and its mappings are not yet fully verified or implemented.

## Source

- Upstream project: Microsoft Fluent UI System Icons
- Source assets, relative to this `symbols/` directory: `source/fluent-ui/`
- License: MIT, as recorded in the source directory's license file

The source collection is not itself the verified AureaOS catalog. Do not assume that a proposed AureaOS name has a matching Fluent asset until the mapping has been checked against the actual files.

## Organization

- `source/` — upstream source assets, preserved separately from AureaOS catalog documentation
- `catalog/` — proposed AureaOS names, categories, naming rules, and future mapping guidance
- `distribution/` — reserved for any future prepared distribution assets; its contents and process are not defined here

## Naming and compatibility

AureaOS plans to expose stable, descriptive names that do not depend on upstream filenames. The names shown in the catalog documents—for example, `folder`, `document-open`, `settings`, and `network-wireless`—are proposed conceptual names. They are examples of intent, not verified Fluent asset names or completed mappings.

When a catalog entry is verified, its record should preserve the exact upstream asset name and source location, plus any verified variant or size metadata. Do not infer this information from a conceptual name.

## Application integration

The planned direction is for applications to use AureaOS catalog names rather than depend directly on source paths. The shared symbol interface and runtime integration have not been implemented. See `catalog/INTEGRATION.md` for the current design status.

## Completeness

The project intends to keep the full Fluent set available for the needs of a modern desktop operating system. The AureaOS catalog may organize and map useful entries, but it must not narrow or discard the source collection merely to keep the catalog small.
