<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Typography Resources

Owns retained font sources, licensing records and current typography guidance. Font selection and rendering must remain consistent across the desktop and first-party applications.

## Structure

- `source/google-fonts/` — retained Noto families and upstream notices.
- `source/adobe/` — retained Source Code Pro and its upstream notice.
- `generated/` — reserved for validated subsets and runtime font packages.

## Current development support

| Family | Role | License |
|---|---|---|
| Noto Sans | Temporary general UI fallback | SIL Open Font License 1.1 |
| Noto Color Emoji | Temporary emoji coverage | SIL Open Font License 1.1 |
| Noto Sans Symbols 2 | Supplementary Unicode coverage | SIL Open Font License 1.1 |
| Noto Sans Math | Mathematical glyph coverage | SIL Open Font License 1.1 |
| Source Code Pro | Temporary monospaced/developer fallback | SIL Open Font License 1.1 |

Keep upstream notices beside each family. These are retained development resources, not a final AureaOS typography identity or a confirmed production font stack.

## Current scope

Aurea Sans is a possible future original type family. Its design and production are explicitly deferred; it is not a prerequisite for the current Human Interfaces consolidation and must not block desktop implementation.

For the current phase:
- Preserve the existing font files and their licenses unchanged.
- Use the retained families only as development support where needed.
- Keep font choice and fallback behavior explicit in consuming components.
- Validate glyph coverage, readability and scaling before runtime packaging.
- Do not rename or relicense third-party fonts as AureaOS artwork.

Revisit an original AureaOS type family only when typography becomes an active product milestone.
