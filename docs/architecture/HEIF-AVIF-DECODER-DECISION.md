<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# HEIF / HEIC and AVIF decoder — integration decision

- Date: 2026-11-24
- Status: **Conditional candidate; implementation not yet approved**
- Owner: `components/aurea-media`
- Scope: single, still-image wallpaper decoding only
- Italian companion: [HEIF-AVIF-DECODER-DECISION-it.md](HEIF-AVIF-DECODER-DECISION-it.md)

## Product boundary

AureaOS has approved HEIF/HEIC and AVIF as still-image wallpaper formats in the official media registry. Approval of the formats does not approve a decoder dependency or claim that these formats currently work.

The adapter must:
- decode only the primary still image;
- reject image sequences and multi-image containers in the first implementation;
- never expose video or sequence playback;
- return owned, tightly packed RGBA8 pixels;
- run only on the existing bounded media worker;
- keep all native codec details out of the compositor and Vulkan renderer.

## Candidate technology

The current candidate is **libheif**, accessed through the safe Rust wrapper **libheif-rs 3.0.0**.

Evidence reviewed:
- libheif uses one decoding API for HEIF and AVIF and supports codec implementations through separate plugins.
- libheif-rs is an MIT-licensed safe wrapper. Its current documentation requires libheif development headers version 1.17 or newer and reports Rust MSRV 1.82.
- The wrapper exposes context security limits with API feature `v1_19`, total-memory limits and sequence detection with `v1_20`.
- Debian 13 Trixie provides libheif 1.23.x, `libheif-dev`, and separate decoder plugin packages.

References:
- https://github.com/strukturag/libheif
- https://docs.rs/libheif-rs/3.0.0/libheif_rs/
- https://docs.rs/libheif-rs/3.0.0/libheif_rs/struct.SecurityLimits.html
- https://docs.rs/libheif-rs/3.0.0/libheif_rs/struct.HeifContext.html
- https://packages.debian.org/trixie/libheif-dev
- https://packages.debian.org/trixie/libheif-plugin-libde265
- https://packages.debian.org/trixie/libheif-plugin-dav1d

## Dependency footprint proposal

Use decoder-only runtime plugins:
- HEIC / HEVC still images: `libheif-plugin-libde265`.
- AVIF still images: `libheif-plugin-dav1d` (or `libheif-plugin-aomdec` only if Debian compatibility testing demonstrates a concrete reason to prefer it).

Do not install encoder plugins, the `libheif-plugins-all` metapackage, FFmpeg decoder plugins, or video playback libraries for this feature. The exact runtime package dependency graph must be confirmed from Debian package metadata before packaging.

Build-time dependency:
- `libheif-dev` (plus the Rust wrapper's pkg-config discovery requirements).

Runtime dependency:
- `libheif1` and only the selected decoder plugins and their transitive runtime libraries.

Fedora package names and versions remain **not verified**. Do not infer local workstation availability from GitHub Actions or Debian metadata.

## Security and maintenance gate

The upstream libheif README reviewed on 2026-11-24 reports that the project is maintained by a single independent developer with limited recurring funding and that 89 security advisories required investigation, fixes and releases during 2026.

This is a material parser-risk signal. It does not automatically disqualify libheif, but it prevents unconditional adoption. Before production use:
- review the current upstream security policy and advisories;
- establish how Debian security updates reach the AureaOS image;
- keep the decoder isolated behind the Aurea Media API;
- ensure malformed files cannot reach the render thread;
- consider a restricted helper process if threat modelling does not accept an in-process native parser.

## Required decode sequence

1. Enforce the existing compressed-input ceiling before passing bytes to libheif.
2. Create a context and apply security limits **before parsing** the untrusted container.
3. Set maximum image pixels to the caller's `DecodeLimits.max_pixels()`.
4. Set the libheif context total-memory ceiling to the caller's `DecodeLimits.max_output_bytes()` (or a stricter documented working-memory budget).
5. Parse from the borrowed input buffer; retain the buffer for the context lifetime.
6. Reject `has_sequence()` and reject anything other than exactly one top-level image.
7. Obtain the primary image handle and validate width, height, pixel count, per-axis limit and RGBA output length using checked arithmetic before decoding.
8. Decode to 8-bit interleaved RGBA. HDR source preservation is outside wallpaper policy v1.0.
9. Validate plane presence, row stride and total readable bytes. Copy row-by-row into a fallibly allocated, tightly packed RGBA8 vector; never assume native stride equals width × 4.
10. Drop image, handle and context before returning the owned Rust buffer.

The wrapper feature set must include at least `v1_20` to expose the required security and sequence APIs. Do not enable the crate's `latest` feature implicitly; select the minimum required API surface deliberately.

## Container identification

Identify the ISO-BMFF `ftyp` major and compatible brands, not the filename extension.

- AVIF still-image brands include `avif`; reject the sequence brand `avis`.
- HEIC still-image brands include `heic` and `heix`; reject the sequence brand `msf1`.
- Do not accept a generic `mif1` brand alone as proof of HEIC/AVIF support.
- Validate box length, integer overflow and compatible-brand alignment before reading brand fields.
- Signature detection is only dispatch. libheif parsing, sequence checks and decoder validation remain mandatory.

## CI and Debian compatibility

The existing Rust CI build runs on Ubuntu 24.04 and currently does not install libheif. Do not add a crate feature that requires libheif 1.20+ to that job without first proving the runner's native library version.

Before enabling the adapter in the default workspace build:
1. Add a dedicated Debian 13 Trixie compatibility job with `libheif-dev`, `libheif-plugin-libde265`, `libheif-plugin-dav1d`, and pkg-config.
2. Verify the exact `libheif-rs` / `libheif-sys` build and link behavior against Debian's installed library, not a vendored or embedded copy.
3. Verify the same API/ABI path on the Fedora development host when available.
4. Confirm the Debian runtime image includes only the selected runtime packages.
5. Add small, licensed or generated HEIC and AVIF still-image fixtures plus malformed, truncated, over-limit, multi-image and sequence fixtures.
6. Run the normal workspace, DRM/Vulkan, security-audit and media tests after the grouped change.

## Current implementation status — 24 November 2026

- Added a private HEIF/AVIF adapter under `aurea-media`, backed by optional `libheif-rs = 3.0.0` with only the `v1_20` API feature enabled.
- `aurea-media` exposes the opt-in Cargo feature `heif`; `aurea-compositor` forwards it through its own `heif` feature. Neither feature is enabled by default, so ordinary workspace builds do not acquire a native libheif requirement.
- The adapter applies input, pixel, dimension and native decoder memory limits; rejects sequence/multi-image input; verifies the requested format against ISO-BMFF brands; and copies the RGBA plane row-by-row into owned tightly packed memory.
- A dedicated Debian 13 CI job installs libheif development headers and selected decoder plugins, then compiles/tests Aurea Media with `heif` enabled. Its first run exposed a Rust visibility error and Debian's distro Rust 1.85 being too old for the resolved `image` crate (MSRV 1.88). Both causes have been addressed in source/workflow, but the corrected run is still required.
- Parser security/maintenance review, actual HEIC/AVIF fixture tests, Fedora package parity and runtime package manifest review remain open gates. This is an opt-in implementation under validation, not yet production approval.


## Production desktop integration — 25 November 2026

- `aurea-desktop` now exposes an opt-in `heif` feature that forwards to `aurea-compositor/heif`; the default development workspace remains independent of native libheif.
- The Debian Live build hook explicitly enables `--features heif` when compiling the production desktop binary.
- The target image installs `libheif1`, `libheif-plugin-libde265` and `libheif-plugin-dav1d` as runtime-only decoder packages. `libheif-dev` is used during compilation and purged before the image is finalized.
- Debian 13 CI now checks the full `aurea-desktop` feature graph with HEIF enabled, not only the standalone media crate.
- This closes the source-to-production-build wiring gap; it does not close the parser security, licensing, real-fixture or physical-hardware validation gates.
