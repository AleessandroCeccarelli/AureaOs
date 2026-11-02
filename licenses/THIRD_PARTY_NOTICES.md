# Third-Party Notices

This file is the project register for third-party code, assets, fonts, icon sets, and other components included in AureaOS or referenced by the project.

The original AureaOS project content is licensed under the GNU General Public License v3.0 (GPL-3.0), unless otherwise stated. That license does not automatically cover third-party software, fonts, assets, or vendor code. Each third-party component must be reviewed, documented, and tracked here.

This document is meant to be updated over time as the project grows and new dependencies are added.

## Purpose

This file helps us do the following:

- keep a clear record of all third-party components used by the project
- document the license and source for each item
- reduce legal and compliance risk
- make it easier to review and replace dependencies later
- keep the repository transparent and professional

## How to update this file

Whenever a new dependency is added, the maintainer or contributor must add a new entry before merging the change.

Each entry should include:

- component name
- version or revision
- author or upstream project
- source URL
- license name
- where it is used in the repository
- purpose (UI, build tool, font, asset, runtime dependency, etc.)
- review status
- notes

If a component is only a runtime dependency from Debian or another package manager, it may be tracked in the package manifests instead of being copied into the repository. This document is primarily for embedded, vendored, copied, or directly included third-party content.

## Current third-party inventory

| Component | Version / Revision | Source | License | Used in | Purpose | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| Microsoft Fluent UI Symbols | project assets / source package | https://github.com/microsoft/fluentui-system-icons | MIT License | aurea-human-interfaces/symbols/source/fluent-ui | Symbol set used as design reference and source assets | Included as source asset. License is MIT. |
| Google Noto Color Emoji | upstream font package | https://github.com/googlefonts/noto-emoji | SIL Open Font License 1.1 | aurea-human-interfaces/fonts/source/google-fonts/noto-color-emoji | Emoji font source | Included as a font source asset. License is OFL 1.1. |
| Debian GNU/Linux Trixie packages | Debian package versions from official repositories | https://www.debian.org/ | Debian licenses, upstream package licenses | debian/packages/ | Runtime system foundation for the AureaOS base | These are not stored in this repository as source code; they are build-time and runtime dependencies. |

## Notes on license handling

- The GPL-3.0 license applies to the original AureaOS project work unless another file clearly states a different license.
- Third-party components keep their own licenses and terms.
- This file is a compliance and transparency record, not legal advice.
- Some package dependencies are not distributed in the repository itself and therefore do not require vendored copies here. They still need to be reviewed and documented in the project manifests and build notes.

## Review rules

Before any third-party material is added to the repository, the following should be checked:

- license compatibility with the project goals
- whether the component is source-available or vendored
- whether redistribution is allowed for this project
- whether attribution is required
- whether any trademark or brand restrictions apply
- whether the component is stable and maintained upstream

## Template for future entries

Use this format when adding a new item:

```text
### Component Name
- Version / Revision: x.y.z or commit SHA
- Author / Upstream: Name
- Source: https://example.com/project
- License: MIT / GPL-3.0 / Apache-2.0 / SIL OFL 1.1 / etc.
- Repository path: path/to/file_or_directory
- Purpose: UI, font, asset, library, runtime dependency, build tool, etc.
- Review status: Reviewed / Pending review / Approved for use
- Notes: Short description, compatibility notes, attribution requirements, or limitations.
```

## Current status

This document is intentionally simple and maintainable. It should be expanded as the project gains new packages, frameworks, fonts, icon sets, bundled assets, and runtime dependencies.

AureaOS must keep this file up to date as the system grows. Every new third-party element should be declared here before it is accepted into the project.
