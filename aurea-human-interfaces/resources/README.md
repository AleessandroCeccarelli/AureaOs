# AureaOS Human Interfaces Resources

This directory holds reusable visual assets for the AureaOS Human Interfaces library and desktop.

- `illustrations/` — authored SVG illustrations for onboarding, empty states, and system messages. Keep wording out of the artwork so products can localize the accompanying UI text.
- `textures/` — subtle, reusable surface treatments. `textures.css` provides theme-aware grid, diagonal-line, and glow patterns using the AureaOS color tokens.
- `wallpapers/` — desktop background collections and their attribution or usage notes. This is the canonical wallpaper location.

Keep each collection's README and licensing or attribution details with its assets. Shared colors and glass treatments belong in `../themes/`, not as baked-in images here. Avoid keeping duplicate copies of an asset in another folder.

The illustrations in this directory are decorative examples and should be paired with accessible UI text. Import `textures/textures.css` after the shared theme tokens before using its `.aurea-texture-*` classes.
