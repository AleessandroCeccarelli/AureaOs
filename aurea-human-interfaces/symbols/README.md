<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Symbol Sources and Catalog

Separates third-party reference symbols from the future stable AureaOS semantic symbol catalog.

## Structure

- `source/fluent-ui/` — curated Microsoft Fluent UI System Icons subset.
- `catalog/` — proposed AureaOS names, categories, mappings and integration contract.
- `generated/` — reserved for optimized or packaged outputs.

## Current inventory

The retained subset contains 89 concepts with Regular and Filled 24 px SVG variants where supplied: 178 SVG files total. It is curated, not a mirror of the complete upstream Fluent collection.

The upstream MIT license is retained at `source/fluent-ui/LICENSE`. These remain Microsoft Fluent UI assets, not original AureaOS artwork.

## Catalog status and destination

The catalog documents naming and functional categories. It is not yet a complete verified mapping or implemented runtime API. Each mapping must be checked against an actual retained source file and record its exact upstream filename and available variant.

Applications should eventually request symbols through a shared AureaOS API using stable semantic names, not upstream paths. The API, mapping manifest, variant fallback, logical sizing and runtime loader remain future work.

Keep the subset curated by concrete product requirements; expand it when a real interface need justifies a new source asset.
