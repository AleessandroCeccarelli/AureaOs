# AureaOS Symbol Categories

## Purpose

This document defines the proposed functional categories for organizing entries in the AureaOS symbol catalog.

The category names and examples below describe catalog concepts. Example symbol names are not verified against, or mapped to, specific Fluent assets in `../source/fluent-ui/` (relative to this `catalog/` directory).

## Categories

### System and Settings

Conceptual examples for system configuration, preferences, maintenance, and administration:

- `settings`
- `system-update`
- `security`
- `accessibility`

### Files and Documents

Conceptual examples for file management, document handling, storage, and file operations:

- `folder`
- `folder-open`
- `document`
- `document-open`
- `document-save`
- `drive`

### Applications and Windows

Conceptual examples for launching, managing, and interacting with applications and windows:

- `app`
- `app-grid`
- `window`
- `window-close`
- `window-minimize`
- `window-maximize`

### Connectivity and Devices

Conceptual examples for network connections, wireless communication, peripherals, and hardware:

- `network-wireless`
- `network-wired`
- `bluetooth`
- `device`
- `usb`

### Accessibility

Conceptual examples for accessibility features and assistive interaction:

- `accessibility`
- `contrast`
- `hearing`
- `text-size`
- `keyboard`

### Notifications and Status

Conceptual examples for notifications, system states, warnings, and informational feedback:

- `notification`
- `status-info`
- `status-success`
- `status-warning`
- `status-error`

### Multimedia and Controls

Conceptual examples for audio, video, playback, display, and media controls:

- `media-play`
- `media-pause`
- `media-stop`
- `volume`
- `brightness`

### Actions and Navigation

Conceptual examples for common actions, navigation, editing, and interface controls:

- `add`
- `remove`
- `edit`
- `delete`
- `search`
- `arrow-left`
- `arrow-right`
- `chevron-down`

## Categorization rules

- Each verified catalog entry should have one primary category.
- Additional tags may be used when an entry serves multiple purposes.
- Categories describe function, not visual appearance.
- New categories may be introduced when they represent a distinct and useful functional area.
- Existing categories should be reused where appropriate to avoid unnecessary fragmentation.

## Completeness

These proposed categories are intended to cover the needs of a modern desktop operating system and its applications. They describe organization, not a claim that all example names or mappings are implemented.

The project intends to keep the full Fluent set needed by a modern operating system available. Catalog categories must not be used to exclude useful source symbols merely to keep the catalog small.
