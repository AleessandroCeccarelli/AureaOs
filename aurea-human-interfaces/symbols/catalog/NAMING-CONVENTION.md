# AureaOS Symbol Naming Convention

## Purpose

This document defines the naming rules for symbols exposed by the AureaOS symbol catalog.

The goal is to provide clear, consistent, and stable names for use across the operating system and its applications.

## General Rules

Symbol names must:

* Use lowercase English words
* Use hyphens to separate words
* Describe the symbol’s meaning or function
* Follow common desktop and Linux terminology where appropriate
* Remain independent of the original Fluent asset filenames
* Avoid unnecessary abbreviations
* Remain stable once published

## Semantic Naming

Names should describe the intended meaning or action rather than the visual appearance of the icon.

Preferred examples:

* folder
* document-open
* document-save
* settings
* network-wireless
* window-close

Avoid names based only on appearance, such as:

* blue-folder
* three-lines
* round-cross

## Actions and Objects

Use established action-object naming where appropriate:

* document-open
* document-save
* window-close
* media-play
* media-pause

For common objects with an unambiguous meaning, use the object name alone:

* folder
* printer
* calendar
* camera

## Variants

Variants must not be included in the base symbol name.

For example, settings identifies the symbol, while regular or filled identifies its variant.

## Source Mapping

Each catalog entry must preserve the original Fluent asset name and map it to the corresponding AureaOS symbol name.

Changing an upstream filename must not require changing the public AureaOS name.

## Consistency

Before adding a new symbol name, check whether an existing catalog entry already represents the same meaning.

Synonyms should be avoided unless they represent genuinely distinct concepts or actions.