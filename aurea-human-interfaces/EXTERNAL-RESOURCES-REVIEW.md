<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# External Resources Review

This document tracks third-party resources that may help AureaOS. It is a research register, not an approval to import assets. The directory `external-resources/` is intentionally kept empty until a concrete resource has passed review.

## Selection principles

- Prefer permissive, redistribution-friendly licenses such as MIT, ISC or Apache-2.0.
- Read the actual upstream license at the exact revision being considered; do not rely only on a catalog or search result.
- Preserve copyright, license and NOTICE files as required.
- Record the upstream repository, pinned release or commit, individual asset paths, authors where available, modifications and intended use.
- Exclude third-party brand marks unless the relevant trademark owner separately authorizes the use.
- Do not duplicate AureaOS artwork or the curated Fluent UI subset without a demonstrated gap.
- Keep third-party assets separate from original AureaOS assets. The repository's GPL license does not relicense them.

## Candidates worth keeping on the radar

| Resource | Upstream license | Potential value | Current decision |
|---|---|---|---|
| [Tabler Icons](https://github.com/tabler/tabler-icons) | MIT | Broad set of generic interface symbols; useful only to fill semantic gaps not covered by AureaOS artwork or the existing Fluent UI subset. | **Best fallback candidate.** Do not import the full pack; select individual generic SVGs only after visual and license review. Preserve MIT notice. |
| [Lucide](https://github.com/lucide-icons/lucide) | ISC; the upstream LICENSE also includes a separate MIT notice for Feather-derived icons. | Consistent outline symbols and a useful source for missing basic UI concepts. | **Secondary fallback.** Preserve the applicable ISC and MIT notices; verify provenance for each selected asset. |
| [Phosphor Icons](https://github.com/phosphor-icons/core) | MIT | Multiple weights can help when a specific visual hierarchy is needed. | **Optional research source.** Its multiple weights may introduce visual inconsistency; do not mix styles without a deliberate design decision. |
| [Folder11](https://github.com/icon11-community/Folder11) | No formal repository-wide license identified. A contributor replied to AureaOS issue [#214](https://github.com/icon11-community/Folder11/issues/214) with a conditional willingness to permit use, modification and redistribution of base folder designs, excluding third-party logos and marks. | Possible reference/source for selected generic folder artwork. | **Not cleared for bulk import.** Keep the permission discussion and attribution link. Before redistributing any particular SVG, confirm that the exact file is a base folder design covered by the permission and that it contains no third-party mark or unverified borrowed artwork. |

## Not selected for now

- Large icon packs that substantially duplicate the existing AureaOS and Fluent UI collections.
- Brand-logo libraries: an open-source file license does not itself grant trademark permission.
- Collections with share-alike or attribution obligations that add complexity without a concrete product need.
- Whole desktop themes or icon themes: AureaOS has its own visual identity and does not adopt external desktop themes.

## Current repository context

AureaOS already maintains:
- Original application, system and Fides folder artwork under `icons/`.
- A curated Microsoft Fluent UI System Icons subset under `symbols/source/fluent-ui/`, with its MIT license retained.
- Fonts, cursors, illustrations, wallpapers and sounds in their existing dedicated collections.

For that reason, the next useful contribution from an external icon source should solve a documented missing concept, not expand the collection for its own sake.

## Import checklist

Before copying any file into `external-resources/`:

1. Identify the exact upstream file, author and immutable release/commit.
2. Inspect the file itself for embedded logos, trademarks, raster images or uncertain provenance.
3. Verify the applicable license or written permission covers copying, modification and redistribution in the intended AureaOS distribution.
4. Add the original license/permission record and attribution.
5. Record any modifications and keep the original source separately where appropriate.
6. Review the result against AureaOS visual and accessibility guidelines.

No third-party asset is approved for packaging merely because it appears in this research register.

_Last reviewed: 2026-11-19._
