# Third-Party Notices

This file is the project register for third-party code, assets, fonts, icons, and other components included in AureaOS or referenced by the project.

The original AureaOS project content is licensed under the GNU General Public License v3.0 (GPL-3.0-only), unless otherwise stated. That license does not automatically cover third-party software, fonts, assets, or vendor code. Each third-party component must be reviewed, documented, and tracked here.

This document is intended to be updated over time as the project grows and new dependencies are added.

## Purpose

This file helps us do the following:

- keep a clear record of all third-party components used by the project
- document the license and source for each item
- reduce legal and compliance risk
- make it easier to review and replace dependencies later
- keep the repository transparent and professional

## Current third-party inventory

| Component | Version / Origin | Source | License | Used in | Purpose | Status |
| --- | --- | --- | --- | --- | --- | --- |
| Microsoft Fluent UI Symbols | upstream symbol source package | https://github.com/microsoft/fluentui-system-icons | MIT | `aurea-human-interfaces/symbols/source/fluent-ui` | Symbol set used as design reference and source assets | Verified |
| Google Noto Color Emoji | upstream font package | https://github.com/googlefonts/noto-emoji | SIL Open Font License 1.1 | `aurea-human-interfaces/fonts/source/google-fonts/noto-color-emoji` | Emoji font source | Verified |
| Debian GNU/Linux Trixie packages | Debian package versions from official repositories | https://www.debian.org/ | Debian package licenses and upstream package licenses | `debian/packages/` | Runtime system foundation | Verified at package level when included; review required for release bundles |

## Notes on license handling

- The GPL-3.0-only license applies to the original AureaOS project work unless another file clearly states a different license.
- Third-party components keep their own licenses and terms.
- This file is a compliance and transparency record, not legal advice.
- Some runtime dependencies are managed by Debian and are not vendored in this repository. They still need to be reviewed and documented in package manifests and release notes.

## Review rules

Before any third-party material is added to the repository, the following should be checked:

- license compatibility with the project goals
- whether the component is source-available or vendored
- whether redistribution is allowed for this project
- whether attribution is required
- whether any trademark or brand restrictions apply
- whether the component is stable and maintained upstream

## Items still requiring verification

These items are not yet sufficiently documented for full compliance confidence:

- any additional wallpaper or artwork added later without explicit source and license metadata
- any font family or icon pack not yet listed in this file
- any future proprietary or external runtime component introduced into build or release images

## Template for future entries

Use this format when adding a new item:

```text
### Component Name
- Version / Origin: x.y.z or commit SHA / upstream source
- Author / Upstream: Name
- Source: https://example.com/project
- License: MIT / GPL-3.0-only / Apache-2.0 / SIL OFL 1.1 / etc.
- Repository path: path/to/file_or_directory
- Purpose: UI, font, asset, library, runtime dependency, build tool, etc.
- Review status: Verified / Pending review / DA VERIFICARE
- Notes: short description, compatibility notes, attribution requirements, or limitations.
```
