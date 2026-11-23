<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Symbol Integration

## Purpose

Describe the future integration direction for symbols in AureaOS applications. This is a design contract, not an implemented API or runtime loader.

## Intended request model

Applications should request verified AureaOS semantic names rather than upstream Fluent filenames or source paths. Names such as `folder`, `document-open`, `settings`, `network-wireless` and `window-close` are examples only until their mappings are verified.

## Mapping requirements

Each verified mapping records:
- stable AureaOS semantic name;
- exact upstream asset filename and source path, if third-party;
- verified variant;
- source dimensions/metadata where relevant;
- license/provenance reference.

Never infer a mapping from a similar-looking filename. If no retained source is suitable, create original AureaOS artwork or explicitly defer the symbol.

## Variants and sizing

Regular/Filled variants and source sizes must be verified per asset. The future API should define stable logical sizes and a predictable fallback when a variant is unavailable. Do not claim every source asset provides every variant or size.

## Runtime direction

The future shared interface should hide upstream paths from applications and provide stable names, variant selection and runtime loading. API shape, asset packaging, fallback behavior and loader implementation remain to be designed alongside the shared AureaOS UI library.

The source collection remains curated. Expansion is driven by actual shell and application requirements, not a goal to mirror the entire upstream pack.
