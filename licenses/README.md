# AureaOS License Registry

This directory contains the central project records for licensing, attribution, and third-party review.

The original AureaOS project content is released under the GNU General Public License v3.0 (GPL-3.0), unless explicitly stated otherwise in a specific file. This directory is intended to track components that are not covered by that project-level license, including vendored source code, bundled assets, third-party fonts, icon sets, libraries, and runtime dependencies that are reviewed as part of the system.

## Files in this directory

- `THIRD_PARTY_NOTICES.md` — master register of third-party items and licensing information
- `README.md` — overview of how the licensing process works in AureaOS
- `SPDX.md` — optional SPDX-style summary for easier project audit and automation

## Purpose

This directory exists to make the project easier to maintain, review, and audit over time.

It helps us:

- track bundled or copied third-party software
- record where assets and dependencies are used
- maintain a clear review trail for legal and compliance concerns
- document licenses without hiding them inside random files
- support future growth of the OS and its app ecosystem

## Review policy

Before a new dependency, font, asset, or third-party library is introduced, it should be added to `THIRD_PARTY_NOTICES.md` before or alongside the merge that introduces it.

This helps prevent undocumented code or assets from being added to the project without review.

## Scope

This directory is for:

- vendored libraries or source copied into the repo
- bundled assets, fonts, and icons
- third-party design resources
- external code intentionally redistributed with the project
- runtime dependencies that are directly included, mirrored, or distributed as part of the OS images

This directory is not meant to replace official Debian package licensing and package metadata for runtime dependencies managed by Debian or the system package manager. Those should still be tracked in manifests and build documentation where relevant.
