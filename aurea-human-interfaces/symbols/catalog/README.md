<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Symbol Catalog

## Purpose

The catalog defines stable AureaOS names and functional categories for symbols used across the desktop and first-party applications.

It is a design proposal, not yet a complete inventory, verified mapping, or implemented runtime interface.

## Naming convention

Names should be descriptive, consistent with desktop terminology, and independent of upstream filenames. Examples such as `folder`, `document-open`, `settings`, `network-wireless`, and `window-close` illustrate the naming style only.

See `NAMING-CONVENTION.md` for the naming rules and `CATEGORIES.md` for the proposed functional groups.

## Source mapping

Where a Fluent reference asset is used, record its exact upstream filename and source location, including variant and size only when verified. The retained source collection is intentionally curated and is not a complete upstream mirror.

A missing source mapping must not be guessed. If no suitable reference exists, the symbol should be designed as original AureaOS artwork or explicitly deferred.

## Application usage

The intended direction is for applications to request symbols through a shared AureaOS interface and catalog names rather than hard-coded source paths. That interface and its implementation do not yet exist. See `INTEGRATION.md` for current design status.
