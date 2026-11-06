<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Symbol Catalog

## Purpose

The catalog documents proposed AureaOS names, functional categories, and the future mapping from those names to verified Fluent UI System Icons source assets.

The catalog documentation is a design proposal. It is not yet a complete inventory, a verified mapping, or an implemented runtime interface.

## Catalog structure

The catalog is organized by functional categories. See `CATEGORIES.md` for the proposed categories and examples.

Names in that document and in other catalog examples are conceptual proposals. They have not yet been checked or mapped to specific files in `../source/fluent-ui/`.

## Naming convention

Proposed AureaOS names should be:

- descriptive and easy to understand
- consistent with common desktop terminology where appropriate
- independent of upstream filenames
- stable once published

Examples such as `folder`, `document-open`, `settings`, `network-wireless`, and `window-close` illustrate the naming style only. They are not verified asset mappings.

See `NAMING-CONVENTION.md` for the naming rules.

## Variants and sizes

The actual variants and size options must be inventoried from the source assets before catalog entries claim they are available. Labels such as `Regular` and `Filled` may be used as examples of variant concepts, but their availability for any particular symbol is not confirmed by this document.

No exact source dimensions or final logical-size API are established here. Once verified, catalog metadata should record the source variant and dimensions and explain any logical-size mapping used by the UI.

## Source mapping

Each verified catalog entry should map an AureaOS conceptual name to an actual Fluent asset. The mapping should record the exact upstream filename and source location, with variant and size information only where verified.

The source directory, relative to this `catalog/` directory, is `../source/fluent-ui/`. Keep upstream assets there; do not rename, move, or narrow the full source collection as a side effect of catalog work.

## Application usage

The intended direction is for applications to request symbols through a future shared AureaOS interface and use catalog names rather than hard-coded source paths. That interface and its implementation do not yet exist. See `INTEGRATION.md` for the current plan and open implementation decisions.

## Completeness

The project intends to keep the full Fluent set needed for a modern operating system available. Catalog organization and future mappings must not be used to exclude useful source symbols solely to reduce catalog size.
