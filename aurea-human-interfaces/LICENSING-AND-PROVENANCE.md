<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Human Interfaces — Licensing and Provenance

## Licensing policy

Human Interfaces contains original AureaOS work and retained third-party material. Preserve this distinction.

| Material | License policy | Required handling |
|---|---|---|
| Original AureaOS specifications, CSS, SVG, illustrations, wallpapers, cursors and sound effects | GPL-3.0-or-later unless specifically stated otherwise | SPDX identifier and repository-root GPL license |
| Microsoft Fluent UI System Icons | MIT | Preserve upstream copyright and MIT license |
| Noto font families | SIL Open Font License 1.1 | Preserve matching upstream OFL notice |
| Source Code Pro | SIL Open Font License 1.1 | Preserve Adobe notice, reserved-name terms and OFL text |
| Future third-party material | Actual upstream license | Review and record before inclusion |

The repository GPL license does not override or relicense third-party material.

## Notice locations

- Repository GPL text: `/LICENSE`.
- Third-party inventory: `/licenses/THIRD_PARTY_NOTICES.md`.
- Fluent UI: `symbols/source/fluent-ui/LICENSE`.
- Fonts: notices retained beside each family under `fonts/source/`.
- Folder artwork: `icons/source/folders/v5/LICENSE.md`.
- Cursor artwork: `cursors/LICENSE.md`.
- Original sound library: `resources/sounds/source/approved-library/LICENSE.md`.

## File headers

New AureaOS-authored Markdown and text documentation should begin with:

```text
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
```

New authored CSS and SVG sources should use the appropriate SPDX comment form. Machine-readable files should use comments only where their format permits them.

Do not add AureaOS headers to upstream third-party files or alter upstream license texts.

## Provenance

Every new collection should document creator/upstream project, source or generation method, version/date where known, format, license, whether files are source/preview/runtime output, and any conversion procedure.

AI-assisted artwork must be described honestly. Do not claim human photography, manual illustration or exclusivity beyond what is known.

## Generated assets

Generated outputs inherit the source license unless a documented legal reason establishes otherwise. Keep outputs separate from editable masters. Changing a file extension is not format conversion.

Before packaging, verify format, metadata, dimensions, accessibility implications and target integration. Record validation rather than marking planned work complete.

This is a project compliance guide, not legal advice.
