<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Symbol Sources and Catalog

Separates retained third-party reference symbols from the future AureaOS semantic symbol catalog. This collection is distinct from original application identity artwork in `../icons/`.

## Structure

- `source/fluent-ui/` — curated Microsoft Fluent UI System Icons subset.
- `catalog/` — AureaOS naming, categories, source mappings and integration contract.
- `generated/` — reserved for optimized or packaged outputs.

## Current inventory

The retained subset contains 89 concepts and 178 SVG files: Regular and Filled 24 px variants where supplied. It is curated, not a mirror of the complete upstream Fluent collection.

The upstream MIT license is retained at `source/fluent-ui/LICENSE`. These remain Microsoft Fluent UI assets, not original AureaOS artwork. Preserve the upstream license and provenance; the repository GPL license does not relicense this collection.

## Catalog and runtime status

The catalog currently documents naming conventions, functional categories and intended integration. It is not yet a complete verified mapping or implemented runtime API.

Before a mapping is approved, verify the exact retained source filename, path, variant and provenance. Do not infer a mapping from a similar-looking name. If no retained source is suitable, create original AureaOS artwork or explicitly defer the symbol.

Applications should eventually request symbols through a shared AureaOS interface using stable semantic names rather than upstream paths. The API, verified mapping manifest, variant fallback, logical sizing, packaging and runtime loader remain future implementation work.

Keep the source subset curated by concrete shell and application requirements; do not expand it merely to mirror the upstream collection.
