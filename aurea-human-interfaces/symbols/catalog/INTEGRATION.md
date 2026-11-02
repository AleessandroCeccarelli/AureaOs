# AureaOS Symbol Integration

## Purpose

This document defines how AureaOS applications access and use symbols from the shared symbol catalog.

The integration interface provides a consistent way to request symbols without depending on the original Fluent asset filenames or source directory structure.

## Symbol Requests

Applications should request symbols using their stable AureaOS catalog names.

Examples:

* folder
* document-open
* settings
* network-wireless
* window-close

Applications must not reference files directly from source/fluent-ui.

## Variants

The symbol interface should support the available variants, including:

* Regular
* Filled

Applications may specify a variant when the visual context requires it. When no variant is specified, the interface should apply the default defined by the catalog.

## Sizes

The symbol interface should support the sizes available in the catalog.

Applications should request symbols using logical size values where possible, rather than relying on hard-coded source asset dimensions.

## Catalog Mapping

Each symbol request must resolve through the catalog to the appropriate Fluent asset.

The mapping should preserve:

* AureaOS symbol name
* Original Fluent asset name
* Variant
* Size
* Source asset location

## Consistency and Compatibility

The symbol interface should provide stable names and predictable behavior across AureaOS applications.

Changes to source assets or their organization should not require application code to change, provided the public catalog names remain stable.

## Future Implementation

The concrete API, asset packaging format, and runtime loading mechanism will be defined during implementation of the AureaOS shared UI library.