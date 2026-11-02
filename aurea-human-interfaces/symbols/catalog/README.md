# AureaOS Symbol Catalog

## Purpose

The symbol catalog defines the names, variants, and mappings used to access icons throughout AureaOS and its applications.

It provides a consistent interface between the original Fluent UI System Icons assets and the user interfaces built on top of them.

## Catalog Structure

The catalog will organize symbols by functional category, including:

* System and settings
* Files and documents
* Applications and windows
* Connectivity and devices
* Accessibility
* Notifications and status
* Multimedia and controls
* Common actions and navigation

## Naming Convention

Symbol names must be:

* Descriptive and easy to understand
* Consistent with common desktop and Linux terminology where appropriate
* Independent of the original Fluent filenames
* Stable for application developers

Examples:

* folder
* document-open
* settings
* network-wireless
* window-close

## Variants and Sizes

The catalog will record the available Fluent variants, including Regular and Filled, and the supported sizes.

The selected variant should reflect the symbol’s role and visual context. Applications should not need to know the original asset filename to request a symbol.

## Source Mapping

Each catalog entry will map its AureaOS symbol name to its original Fluent asset, including its variant and size.

Original source assets must remain preserved in source/fluent-ui.

## Application Usage

Applications should access symbols through the shared AureaOS symbols interface.

They should use catalog names rather than directly referencing source asset paths. This keeps the application interface stable if the underlying assets or directory structure change.

## Completeness

The catalog is intended to provide comprehensive icon coverage for a modern desktop operating system.

Useful symbols should not be excluded solely to reduce catalog size.