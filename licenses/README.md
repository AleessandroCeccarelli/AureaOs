# AureaOS License Registry

This directory contains the central project records for licensing, attribution, and third-party review.

The original AureaOS project content is released under the GNU General Public License v3.0 (GPL-3.0-only), unless explicitly stated otherwise in a specific file. This directory tracks third-party code, assets, fonts, icons, and other components that are not covered by the project’s original license statement.

## Files in this directory

- `THIRD_PARTY_NOTICES.md` — master register of third-party items and licensing information
- `README.md` — overview of the licensing and review process
- `SPDX.md` — SPDX-style summary for project audit and maintenance

## Purpose

This directory exists to make the project easier to maintain, review, and audit over time.

It helps us:

- track bundled or copied third-party software
- record where assets and dependencies are used
- maintain a clear review trail for legal and compliance concerns
- document licenses without hiding them inside random files
- support future growth of the OS and app ecosystem

## Review policy

Before a new dependency, font, asset, or third-party library is introduced, it should be added to `THIRD_PARTY_NOTICES.md` before or alongside the change that introduces it.

This helps prevent undocumented code or assets from being added to the project without review.

## Scope

This directory is for:

- vendored libraries or source copied into the repo
- bundled assets, fonts, and icons
- third-party design resources
- external code intentionally redistributed with the project
- runtime dependencies that are directly included, mirrored, or distributed as part of project assets or system images

This directory is not meant to replace official Debian package licensing metadata for runtime dependencies managed by the system package manager. Those dependencies still need review and documentation in the relevant manifests and build notes.

## License relationship

The project-level license applies to the original AureaOS work. Third-party materials keep their own licenses and must remain distinctly tracked and attributed.

The primary project license is:

- `GPL-3.0-only`

This is the canonical project identifier used in SPDX-related documentation unless a specific file states otherwise.
