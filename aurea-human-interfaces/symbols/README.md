<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Symbols

## Purpose

This directory contains a small, curated reference subset of Fluent UI System Icons and the documentation for the future AureaOS symbol catalog.

The upstream collection was intentionally reduced. AureaOS keeps only commonly useful desktop and system symbols, in the 24 px Regular and Filled variants where those variants exist. This is a reference and development resource, not the final proprietary AureaOS symbol identity.

## Source and license

- Upstream project: Microsoft Fluent UI System Icons
- Source assets: `source/fluent-ui/`
- License: MIT, recorded in `source/fluent-ui/LICENSE`

The retained source files remain third-party assets. They must not be presented as original AureaOS artwork.

## Organization

- `source/` — the limited upstream reference assets
- `catalog/` — proposed AureaOS names, categories, naming rules, and mapping guidance
- `generated/` — reserved for future generated assets

## Naming and compatibility

Applications should ultimately request symbols through a shared AureaOS interface and stable AureaOS catalog names, rather than depend directly on upstream filenames. The shared interface and verified mappings have not yet been implemented.

Catalog examples are proposals until each entry is checked against a retained source asset. Record the exact upstream filename and available variant when a mapping is verified.

## Curation policy

Keep the source collection focused on common operating-system needs. Do not re-import the full upstream pack by default. Add a symbol only when a concrete AureaOS interface requirement justifies it; prefer original AureaOS artwork for distinctive product and brand identities.
