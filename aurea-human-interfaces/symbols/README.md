# AureaOS Symbols

## Purpose

The symbols library provides the iconography used throughout AureaOS and its applications.

AureaOS adopts Microsoft Fluent UI System Icons as its symbol source. The complete available icon set is retained so that the operating system and applications can use the symbols they need without unnecessary restrictions.

## Source

* Upstream: Microsoft Fluent UI System Icons
* Local source assets: source/fluent-ui
* License: MIT

## Organization

The symbols library is organized into:

* source/ — original upstream assets, preserved as received
* catalog/ — AureaOS symbol catalog, names, variants, and mappings
* distribution/ — selected and prepared assets for system and application use

## Naming and compatibility

AureaOS will expose stable, descriptive symbol names independent of upstream filenames.

Names should follow common desktop and Linux conventions where appropriate, such as folder, document-open, settings, and network-wireless.

The catalog will preserve mappings to the original Fluent asset names and identify available variants and sizes.

Application integration

Applications should request symbols through the shared AureaOS symbols interface rather than referencing source asset paths directly.

This allows the underlying assets to evolve while keeping application interfaces consistent.

## Completeness

The catalog is intended to cover the needs of a modern desktop operating system, including system settings, files, applications, windows, connectivity, accessibility, notifications, multimedia, and common actions.

No useful symbol is excluded merely to keep the catalog small.