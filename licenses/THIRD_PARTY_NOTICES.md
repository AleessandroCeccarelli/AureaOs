<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Third-Party Notices

This file records third-party code, assets, fonts, icons, and other components included in AureaOS. Original AureaOS project content is licensed under GPL-3.0-or-later unless otherwise stated. That license does not automatically cover third-party material.

## Current third-party inventory

| Component | Origin | Source | License | Repository path | Purpose |
| --- | --- | --- | --- | --- | --- |
| Microsoft Fluent UI System Icons | Microsoft | https://github.com/microsoft/fluentui-system-icons | MIT | `aurea-human-interfaces/symbols/source/fluent-ui/` | Curated 24 px Regular/Filled reference symbols |
| Noto Sans | Google Fonts | https://github.com/notofonts/latin-greek-cyrillic | SIL Open Font License 1.1 | `aurea-human-interfaces/fonts/source/google-fonts/noto-sans/` | Temporary sans-serif fallback |
| Noto Color Emoji | Google Fonts | https://github.com/googlefonts/noto-emoji | SIL Open Font License 1.1 | `aurea-human-interfaces/fonts/source/google-fonts/noto-color-emoji/` | Temporary emoji glyph coverage |
| Noto Sans Symbols 2 | Google Fonts | https://github.com/notofonts/symbols | SIL Open Font License 1.1 | `aurea-human-interfaces/fonts/source/google-fonts/noto-sans-symbols-2/` | Supplementary Unicode symbols |
| Noto Sans Math | Google Fonts | https://github.com/notofonts/math | SIL Open Font License 1.1 | `aurea-human-interfaces/fonts/source/google-fonts/noto-sans-math/` | Mathematical glyph coverage |
| Source Code Pro | Adobe | https://github.com/adobe-fonts/source-code-pro | SIL Open Font License 1.1 | `aurea-human-interfaces/fonts/source/adobe/source-code-pro/` | Temporary monospaced developer fallback |
| Debian GNU/Linux packages | Debian | https://www.debian.org/ | Varies by package | `debian/packages/` | Runtime system foundation; review at package/release level |
| Smithay 0.7.0 | Smithay project | https://github.com/Smithay/smithay | MIT | Cargo dependency (`aurea-compositor`) | Wayland compositor framework; fetched from crates.io and pinned in `Cargo.lock` |
| input 0.9.1 | Smithay project | https://github.com/Smithay/input.rs | MIT | Cargo dependency (`aurea-compositor`, optional native DRM backend) | Rust libinput bindings; `libinput_1_21` API enabled for DWTP |
| serde | Serde project | https://github.com/serde-rs/serde | MIT OR Apache-2.0 | Cargo dependency (`aurea-desktop-api`, `aurea-compositor`) | Serialization of persistent touchpad preferences |
| TOML 0.8 | TOML project | https://github.com/toml-rs/toml | MIT | Cargo dependency (`aurea-compositor`) | Durable touchpad preference file format |
| sha2 0.10 | RustCrypto | https://github.com/RustCrypto/hashes | MIT OR Apache-2.0 | Cargo dependency (`aurea-compositor`) | Privacy-preserving stable physical input identity hash |

## License handling

- Third-party components retain their own licenses and notices.
- The license files shipped with each retained font family and the Fluent source directory must remain alongside those assets.
- This register is a compliance aid, not legal advice.
- Runtime dependencies managed by Debian must be reviewed through package metadata and release records.

## Review rules

Before adding third-party material, review its license, redistribution terms, attribution requirements, source provenance, and relevance to AureaOS. Avoid importing broad asset collections without a concrete product need.
