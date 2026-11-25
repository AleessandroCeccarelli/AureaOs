<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Dependency Register

## Purpose and authority

This is the mandatory engineering record for external libraries, system packages, Rust crates, tools and runtime services used or considered by AureaOS.

**Debian GNU/Linux 13 (Trixie), amd64, is the target platform and compatibility authority. Fedora is a development host only.** Fedora availability is useful, but never sufficient evidence for accepting a dependency.

## Mandatory selection order

1. Confirm availability and supportability on Debian 13 Trixie amd64.
2. Record exact Debian binary/source package names and versions.
3. Confirm required headers, libraries, tools and runtime components in supported Debian repositories.
4. Check Fedora development packages separately. Fedora-only APIs, paths or packaging assumptions are not acceptable.
5. Review upstream maintenance, release history, security advisories, license, ABI stability, architecture support and transitive footprint.
6. Prefer existing Linux facilities or already-approved AureaOS components when they meet the requirement.
7. Record the decision and evidence here before adding a dependency to production code or packaging.

A dependency is not approved merely because it appears in Cargo.toml, a design document, prototype, CI image or developer workstation.

## Status vocabulary

- **Adopted** — selected for architecture; this does not claim complete integration or runtime validation.
- **Approved** — reviewed for the stated scope and permitted for implementation.
- **Candidate** — promising, but checks remain open; do not integrate yet.
- **Conditional** — usable only after stated conditions are met.
- **Rejected** — not accepted for the stated scope.
- **Needs review** — existing dependency/integration with incomplete evidence or requiring reassessment.

## Required fields for every entry

| Field | Required information |
|---|---|
| Name and purpose | Exact project/library/package and AureaOS capability |
| Type | Debian package, Rust crate, system service, build tool, protocol or asset tool |
| Status and scope | Status above, component and feature scope |
| Debian target | Debian 13 package(s), source package, version and repository/component, if verified |
| Fedora development | Fedora package(s) and version, if verified; otherwise Not verified |
| Rust integration | Crate/version, features, MSRV, native linking/bindgen details where applicable |
| ABI and architecture | ABI/API version, dynamic/static linking, amd64 support and constraints |
| License | SPDX identifier and compatibility review, including material transitive licenses |
| Maintenance and security | Upstream activity, support status, advisories and mitigation |
| Dependency footprint | Runtime/build transitive dependencies, services, disk/memory impact |
| Evidence | Official package metadata, upstream docs, source links, date and exact versions checked |
| Decision | Need, alternatives, constraints and approval date |
| Validation | What was actually built/tested on Debian and Fedora; package availability is not validation |

Unknown values must be marked **Not verified**. Never fill gaps by guessing.

## Current platform baseline

| Technology | AureaOS role | Debian target | Fedora development | Status |
|---|---|---|---|---|
| Debian 13 Trixie amd64 | Distribution base and runtime target | Target platform | Not applicable | Adopted |
| Rust toolchain | First-party implementation language | Exact packaged/toolchain policy to be recorded | Development toolchain | Adopted; version policy needs review |
| Wayland | Desktop protocol/display model | Debian-supported protocol/library packages | Check development packages per feature | Adopted |
| Smithay 0.7 | Wayland compositor framework | Cargo-resolved crate; native dependencies must be Debian-compatible | Development host only | Adopted; transitive security review open |
| Vulkan | Exclusive compositor rendering API | Debian loader/driver/toolchain support per target GPU | Fedora GPU stack for development validation only | Adopted |
| DRM/KMS, udev | Kernel display/device interfaces | Debian kernel and userspace integration | Development host | Adopted |
| libinput | Input device handling | Debian package/version and feature support to verify | Fedora development package to identify | Adopted architecturally; package record incomplete |
| libseat | Session/device access | Debian package/version/backend to verify | Fedora development package to identify | Adopted architecturally; package record incomplete |
| calloop | Event-loop integration | Rust crate and native dependencies to check | Cargo development environment | Adopted |
| systemd | Init, services and system integration | Debian 13 systemd | Not runtime authority | Adopted |
| PipeWire | Multimedia infrastructure | Debian 13 packages/modules per feature to verify | Fedora development packages | Adopted in technical direction; feature review required |
| NetworkManager | Network management | Debian 13 package/service integration | Fedora development package | Adopted in technical direction |
| Flatpak / Flathub | Third-party app delivery | Debian 13 runtime and portal compatibility | Fedora is not compatibility proof | Adopted in product direction; app-by-app review |

This table records architectural choices, not proof that every package is installed, integrated or validated.

## Component-level decisions

### JPEG decoding — libjpeg-turbo / TurboJPEG compatibility API

- **Status:** Implemented as an isolated first milestone in `aurea-media`; CI validation is in progress. Production approval remains conditional on CI, local Fedora host verification, decoder-boundary/security review and runtime packaging review.
- **Purpose:** Decode the initial JPEG wallpaper into bounded RGBA8 pixels without introducing a general-purpose image stack.
- **Debian evidence verified:** Debian 13 Trixie amd64 provides `libturbojpeg0` and `libturbojpeg0-dev`, version `1:2.1.5-4`. The development package depends on the matching runtime package.
- **Fedora development evidence:** GitHub Actions installed `libjpeg-turbo-devel` and `turbojpeg-devel` from Fedora repositories; tested build: `3.1.4.1-2.fc45`. The user's local Fedora package build remains to be checked.
- **Compatibility finding:** The CI probe compiled with `-Wall -Wextra -Werror`, linked and loaded `libturbojpeg.so.0`, and decoded the committed 3840 × 2160 wallpaper successfully on Debian `1:2.1.5-4` and Fedora `3.1.4.1-2.fc45`. Both produced 33,177,600 RGBA8 bytes and opaque alpha. This validates the tested legacy API surface and runtime path in those container images, not every decoder input or the user's local host.
- **Rust binding decision:** No `turbojpeg` / `turbojpeg-sys` crate or bindgen is used. `aurea-media` compiles a narrow Aurea-owned C shim against the distribution's `turbojpeg.h`; private Rust FFI calls only the shim's stable Aurea-owned functions. This avoids copying native enum values or opaque handle layouts into Rust.
- **Proposed integration boundary:** A small Aurea-owned Rust adapter with all unsafe FFI isolated in one module; expose only a safe decode contract returning the existing bounded `WallpaperPixels`. Decode on a bounded worker, never on the render thread. Validate compressed input length, decoded dimensions, checked RGBA allocation, maximum pixel count and total memory budget before allocating.
- **Alternatives reviewed:** `zune-jpeg` 0.5.15 is blocked because upstream reports incorrect colors for valid baseline JPEGs; the identified fix is only in 0.5.16-rc3, a pre-release. `jpeg-decoder` 0.3.2 is pure Rust but upstream describes it as maintenance-mode; Debian Trixie packages 0.3.0. Neither is currently selected as the primary path.
- **Open approval checks:** Rust CI is pending; repeat native package verification on the user's Fedora workstation; confirm the runtime package `libturbojpeg0` is included in the target Debian image and `libturbojpeg0-dev` is available at build time; review current libjpeg-turbo security advisories and BSD-style license notices; test baseline/progressive, grayscale, truncated and malformed inputs; define worker cancellation before exposing arbitrary-file decoding.
- **Native compatibility:** GitHub Actions run `45356390011` passed on Debian 13 Trixie (`1:2.1.5-4`) and Fedora (`3.1.4.1-2.fc45`); both compiled and executed the C probe and decoded the committed wallpaper.
- **Implementation:** `components/aurea-media` is now a workspace member. It contains the safe API, explicit limits, stable errors, JPEG orchestration and a private C shim/Rust FFI boundary. The root lockfile records the existing `cc` build dependency. Rust CI run is pending; do not describe the crate as validated until it completes.
### SVG rasterization — librsvg

- **Status:** Rejected for the current core desktop runtime scope.
- **Reason:** Its dependency chain includes Cairo, GLib, Pango and related components, disproportionate for the current static-icon requirement.
- **Accepted direction:** Keep editable SVG masters in Human Interfaces and generate bounded raster assets using a documented, reproducible build-time toolchain. Record provenance and licensing.
- **Reconsider only** for a concrete runtime SVG requirement and a fresh Debian-first footprint review.

### SQLite

- **Status:** Candidate for local catalog/search metadata; not approved.
- **Debian evidence recorded:** Debian 13 provides SQLite runtime and development packages.
- **Open checks:** Verify FTS5 in the Debian target build; review the Rust binding; define MSRV; check Debian/Fedora package versions, WAL/recovery behavior, license and maintenance.
- **Constraint:** Do not add a database for speculative indexing. Filesystem scanning and reconciliation remain authoritative where applicable.

### Filesystem notifications — notify / native inotify

- **Status:** Conditional; no crate selected.
- **Evidence recorded:** notify 9 documents Linux inotify support and CC0 licensing, but its current MSRV is Rust 1.91. Inspected workspace manifests use edition 2024 and do not declare rust-version.
- **Before approval:** Set AureaOS MSRV policy; verify Debian toolchain compatibility; compare the crate with a small native inotify integration; document overflow/reconciliation behavior.
- **Constraint:** Notifications are low-latency hints, not replacements for authoritative scans and reconciliation.

### Passkeys / FIDO2 / WebAuthn

- **Status:** Needs architecture/dependency review; no crate selected.
- **Evidence recorded:** passkey-rs provides WebAuthn client and CTAP2 authenticator components; Linux FIDO libraries have different transport and licensing trade-offs.
- **Before approval:** Define whether AureaOS needs client, authenticator, CTAP transport or all three; evaluate Debian packages and hardware/udev access; review license, maintenance, security, TPM optionality and privilege boundaries.
- **Constraint:** Do not assume all passkeys are stored in a TPM. Do not add a privileged service or native library before defining security and IPC boundaries.

### Rust dependency security — cgmath via Smithay

- **Status:** Needs review; tracked maintenance/security concern.
- **Evidence recorded:** Smithay 0.7.0 currently brings cgmath 0.18.0, associated with RustSec RUSTSEC-2026-0196 (unmaintained) and RUSTSEC-2026-0197 (unsound).
- **Required action:** Reassess the exact locked graph and upstream resolution before changing versions or applying overrides. Upstream Smithay master was observed replacing cgmath with glam, but master is not automatically a safe or compatible release choice.
- **Constraint:** A green audit job with explicitly allowed warnings is not a clean dependency report.

## Approval checklist

- [ ] Debian 13 Trixie amd64 provides the package or a supportable source-build path.
- [ ] Exact package and upstream versions are recorded.
- [ ] Fedora development packages are identified separately.
- [ ] No Fedora-only API, path or packaging assumption is introduced.
- [ ] ABI/API, linking and runtime packaging are understood.
- [ ] License and material transitive licenses are compatible with AureaOS distribution.
- [ ] Upstream maintenance and security advisories are reviewed.
- [ ] MSRV, Rust edition and toolchain constraints are explicit.
- [ ] Transitive dependency and runtime footprint are justified.
- [ ] Existing Linux facilities and approved components were considered.
- [ ] Decision and evidence are recorded here.
- [ ] Validation claims match tests actually performed.

If a required answer is unknown, keep the dependency Candidate, Conditional or Needs review.

## Change procedure

1. Update this register before or in the same change that introduces the dependency.
2. Update both English and Italian registers.
3. Record reason, exact versions, package names, sources and date.
4. Update the relevant architecture document and development diary when project direction changes.
5. Never claim Debian compatibility based only on Fedora compilation or Ubuntu CI.
6. Do not run builds, formatting, tests or CI without the user's explicit approval under the current workflow rules.

## Maintenance

Review this register whenever a dependency is proposed, added, removed or upgraded; Debian stable package versions/security status change; Rust MSRV/toolchain policy changes; an advisory affects a direct/transitive dependency; or a component changes native ABI, linking or runtime requirements.

This register is the shared source of truth for dependency decisions. When implementation convenience conflicts with Debian compatibility, Debian compatibility wins.

## Universal desktop and multimedia strategy — 24 November 2026

### Product direction
- AureaOS is intended for children, older adults, first-time users, students, creative and office professionals, administrators and developers through one coherent desktop; no age-based editions or assumptions about ability.
- Adopt the product principle: simple by default, capable when needed, accessible from the beginning.
- Accessibility, multimedia and productivity are foundational requirements, not post-release polish.

### Technology evaluations recorded (not added as dependencies)
- PipeWire + WirePlumber: recommended system audio/video routing and session-policy foundation. Debian Trixie package evidence reviewed: PipeWire 1.4.2 and WirePlumber 0.5.8. Actual image inclusion and runtime integration remain to be verified.
- GStreamer 1.x: recommended modular media pipeline for playback, capture, encoding, muxing and streaming. Debian Trixie package evidence reviewed: 1.26.2 family. Codec/plugin licensing and availability must be assessed per feature; do not install every plugin group by default.
- XDG Desktop Portal: recommended application interoperability and permission boundary for Flatpak. Debian Trixie package evidence reviewed: xdg-desktop-portal 1.20.3. AureaOS must implement its own backend for supported interfaces rather than depending on GNOME/KDE desktop backends.
- AT-SPI2: target Linux assistive-technology interoperability. Orca is a real screen-reader validation target. AccessKit is a candidate Rust abstraction for native widget accessibility, not a replacement for AT-SPI2 or a complete toolkit.
- Parley and cosmic-text: candidate Rust text-layout building blocks for shaping, bidi, line breaking and font fallback. Neither is a complete UI toolkit; prototype before selection.
- Native UI toolkit remains undecided. Any candidate must prove Wayland-only operation, Vulkan-compatible rendering, AT-SPI2/AccessKit accessibility, IME, complex text, touch, scaling, performance and Debian packaging.

### Architecture boundaries
- Keep Aurea Media and Aurea Capture as separate responsibilities: image decode returns bounded CPU-owned pixels; capture coordinates authorization/session lifecycle; PipeWire transports streams; GStreamer handles selected pipeline/encoding work; compositor owns final authorized frames and Vulkan presentation only.
- Do not put media codecs, muxing, audio mixing or file writing in the compositor/render thread.
- Do not introduce a monolithic media daemon. Start with libraries/adapters; reassess a restricted helper process for untrusted-media isolation when threat model and client scope justify it.
- New/immature technologies may be prototyped in isolated crates/processes/features, but must not become base-system or boot/session dependencies before measurable benefit, security review, Debian compatibility and rollback path are established.

### Documentation
- Added docs/architecture/UNIVERSAL-DESKTOP-TECHNOLOGY-STRATEGY.md.
- Aligned English and Italian vision and technology-principles documents and linked the new strategy from both README entry points.
- No dependency, Cargo manifest, lockfile, Rust source or system package was changed. No build, format, check, test or CI was run.

### LocalSend Protocol v2.x — cross-device transfer interoperability

- **Status:** Candidate protocol interoperability; no dependency or implementation approved.
- **Purpose:** Local-first transfer of files, photos, audio, video, links and text between AureaOS and Android/iOS/macOS/other desktop clients.
- **Evidence reviewed:** The published protocol repository documents v2.2 REST discovery and HTTPS upload/download flows, receiver approval, per-file tokens and optional SHA-256 integrity verification. The LocalSend project documents cross-platform clients for Linux, macOS, Windows, Android and iOS.
- **Proposed use:** Implement a narrow Rust adapter behind Aurea Link's stable API. Prefer authenticated HTTPS app-to-app mode; discovery is not authentication. Provide manual IP/QR fallback where multicast is blocked. Stream large files with bounded buffers and safe temporary storage.
- **Code reuse:** Protocol interoperability does not authorize copying or bundling the LocalSend application. Before reuse, inspect the protocol repository's current license and maintenance/security posture, then record any implementation crate's exact version, license and transitive graph.
- **Security gates:** explicit pairing/consent, peer identity validation, no silent HTTP downgrade, filename/path validation, size/session/rate limits, hash verification, collision-safe destination, atomic publication, cancellation and cleanup.
- **Debian/Fedora:** no package or crate has been selected. Check Debian 13 availability and the actual Rust/toolchain/runtime requirements before implementation. Fedora-only validation is insufficient.
- **Alternatives:** KDE Connect is an optional interoperability target; AirDrop/AWDL/OWL remains experimental and hardware-dependent, not a release requirement.
- **Constraint:** no always-on daemon, cloud relay, message/contact/notification sync or live media streaming in the first milestone. Keep transfer outside compositor, render thread and Aurea Media decoder.
- **References:** https://github.com/localsend/protocol and https://github.com/localsend/localsend
- **Validation:** No dependency, Cargo manifest or lockfile change; no build or tests.

### Social/online service APIs — provider-specific candidates

- **Status:** Research only. No SDK, OAuth client or provider adapter approved.
- **Purpose:** Share local files/media/text from Fides and Aurea first-party viewers to online services without manual file re-selection.
- **Candidate services:** Telegram, WhatsApp, Facebook, Instagram, Reddit, YouTube and email.
- **Important distinction:** official APIs differ by account class and action. Telegram Bot API is not a personal-user API; WhatsApp Cloud API is business messaging; Facebook Graph publishing is Page-oriented; Instagram publishing targets eligible Professional accounts; Reddit requires OAuth/access approval and written permission for commercial use; YouTube Data API uploads to an authenticated channel.
- **Fallback hierarchy:** direct official API only where allowed → official share/deep link → registered desktop application handoff → unsupported (never private endpoint/scraping).
- **Security:** OAuth/PKCE where supported, least privilege, system credential store, revocation, final user confirmation, no embedded confidential secrets, no silent publishing.
- **References:** official provider docs are recorded in docs/architecture/AUREA-SHARE-ONLINE-INTEGRATIONS.md.
- **Compatibility:** Debian 13 baseline; no GNOME/KDE, X11/XWayland or OpenGL dependency. API adapters stay outside compositor and media decode/render paths.
- **Validation:** no dependency, source code, OAuth registration or tests added.


## Native compositor build prerequisites — preliminary Debian audit (24 November 2026)

### Shader compiler

- **Tool:** `glslangValidator`, invoked by `components/aurea-compositor/build.rs` when the `drm` feature is enabled.
- **Debian 13 evidence:** `glslang-tools` is available for Trixie amd64, version `15.1.0+1.4.309.0-1`; Debian identifies it as the package containing the validator tools.
- **Fedora development package:** `glslang`.
- **Scope:** build-time only. Do not add it to the installed desktop runtime package set unless a runtime use is separately justified.
- **Status:** Debian package availability verified; an actual Debian build has not yet been run.
- **Evidence:** https://packages.debian.org/trixie/amd64/glslang-tools

### Native Smithay / DRM / input development libraries

- **Known integration:** Smithay 0.7 enables `backend_drm`, `backend_gbm`, `backend_libinput`, `backend_udev`, `backend_session_libseat` and `backend_vulkan` under the compositor's `drm` feature.
- **Debian package evidence:** `libinput-dev` exists in Trixie and depends on `libinput10`, `libudev-dev`, `libmtdev-dev` and `libwacom-dev`. Debian's Trixie wlroots source build also lists `libgbm-dev`, `libseat-dev`, `libvulkan-dev` and `libwayland-dev` as build dependencies.
- **Status:** candidate native build package set only; exact direct requirements for Smithay 0.7 and required pkg-config modules must be confirmed from the locked Cargo graph and a Debian build. Do not copy the complete wlroots build-dependency list blindly.
- **Runtime separation:** development headers and shader tools must stay out of the installed runtime image unless a specific runtime package is required.
- **Evidence:** https://packages.debian.org/en/stable/libdevel/libinput-dev and https://packages.debian.org/it/source/trixie/wlroots
- **Validation:** no Debian compositor build has yet been performed; no package has been added to the target runtime manifests.


## rsvg-convert — static SVG asset generation

- **Type:** asset-generation CLI, developer host only; reuses the renderer already required by the cursor-theme builder.
- **Status:** Candidate for desktop exports; not an AureaOS runtime dependency.
- **Debian 13 Trixie:** `librsvg2-bin`, version `2.60.0+dfsg-1+deb13u1`; provides `rsvg-convert`.
- **Fedora 45:** `librsvg2-tools`, version `2.63.2-2.fc45`; provides `rsvg-convert`.
- **Footprint:** the host renderer depends on librsvg and Cairo/GLib/Pango components plus other libraries. This footprint stays on the development/asset-generation host; no SVG library is added to the AureaOS runtime.
- **License:** librsvg is LGPL-2.1-or-later with dependency licenses listed by the distribution packages; further review is required only if the tool or libraries are ever shipped.
- **Use:** offline rasterization of the six approved Trash SVG masters; the exact renderer version is recorded in the export manifest. The compositor does not link this tool.
- **Evidence:** Debian Trixie and Fedora 45 package metadata checked on 24 November 2026.
- **Validation:** the tool is already used by the cursor-theme generator; the new Trash export pipeline has not yet been run and its output has not been inspected. No runtime dependency was added.


### Asset generator: Python standard library

- `tools/assets/png_to_rgba.py` uses only Python 3 standard-library modules (`argparse`, `struct`, `zlib`, `pathlib`); no pip package or OS runtime library is required.
- **Classification:** host-only asset-generation tool; it is not part of Cargo manifests or the AureaOS image.
- **Limits:** non-interlaced RGBA8 PNG, maximum 256 × 256, CRC validation, bounded decompression and PNG filter conversion 0–4.
- **Status:** source committed; not yet run or validated on the laptop.


## Aurea Media — static wallpaper image formats (24 November 2026)

### Rust image crate

- **Status:** Candidate; not yet added to Cargo manifests.
- **Purpose:** decoder adapters for PNG, BMP, GIF, TIFF and WebP; JPEG may remain on the existing TurboJPEG backend during migration.
- **Candidate upstream:** image 0.25.x. Its documented format feature set includes BMP, GIF, JPEG, PNG, TIFF and WebP. Use default-features = false and enable only required format features; do not inherit the default Rayon feature or unrelated codecs.
- **AVIF:** do not assume the default avif feature enables decoding. The documented native AVIF decode path uses avif-native and libdav1d; evaluate it against libheif before selection.
- **Debian 13:** pure Rust codec integration is expected to avoid adding system codec packages for these formats, but the exact locked crate graph, MSRV, license metadata and build behavior remain Not verified.
- **Fedora development:** Cargo dependency; exact host build not run.
- **License:** upstream project is dual-licensed; exact version's Cargo metadata and all enabled transitive licenses must be checked before approval.
- **Security/limits:** decoder dimensions and allocation limits must be enforced before pixel allocation; a bounded compressed-input size alone is insufficient.
- **Decision:** Candidate only. No source, manifest or lockfile changes authorized by this register entry.

### libheif and image decoder plugins

- **Status:** Candidate for HEIC/HEIF and AVIF still-image decoding; not yet linked by AureaOS.
- **Purpose:** ISO-BMFF HEIF container parsing and dispatch to still-image codecs. HEIC uses HEVC image coding; AVIF uses AV1 image coding. This entry does not authorize video playback.
- **Debian 13 Trixie:** package metadata confirms libheif1 and libheif-dev; current metadata surfaced during review reports 1.23.6-2~deb13u1 for the runtime package. Exact amd64 package version must be rechecked at implementation/packaging time because security updates can change it.
- **HEIC decoder plugin:** libheif-plugin-libde265, with runtime dependency libde265-0. Debian describes this as the plugin that decodes HEVC images.
- **AVIF decoder plugin:** choose one supported AV1 decoder plugin, preferably libheif-plugin-dav1d or libheif-plugin-aomdec, after checking the exact Trixie package and transitive license/footprint.
- **Fedora development:** package names and versions not yet verified.
- **Rust integration:** no Rust wrapper selected. Prefer a small audited C ABI shim or a narrowly scoped maintained wrapper; keep all unsafe/native handles inside the adapter.
- **License:** libheif and plugin/codec license combination not yet fully reviewed against GPL-3.0-or-later distribution obligations. Not approved until reviewed.
- **Runtime footprint:** install only the required decoder plugins. Do not install libheif-plugins-all, encoder plugins or FFmpeg plugins for this wallpaper feature.
- **Security:** parse untrusted files off the render thread; enforce image dimension, pixel, output-byte and container item limits; inspect upstream advisories and supported ABI.
- **Evidence:** Debian Trixie package metadata and descriptions for libheif1, libheif-plugin-libde265; https://packages.debian.org/trixie/libheif1 and https://packages.debian.org/trixie/libheif-plugin-libde265
- **Decision:** Optional implementation added behind `aurea-media/heif`; production/runtime package adoption remains gated on CI, security review, fixture coverage and package/license verification.

### JPEG 2000

- **Status:** Deferred candidate.
- **Purpose:** optional macOS interoperability for .jp2/.j2k images.
- **Evidence:** Apple Preview documents JPEG 2000 export, but no Debian 13 decoder package, Rust adapter, license or footprint has yet been approved.
- **Decision:** Do not claim support until a Debian-compatible backend is selected and tested.

### Scope exclusion

No video formats or playback libraries are selected for wallpaper support. GStreamer, FFmpeg video plugins, H.264, HEVC video and AV1 video remain outside Aurea Media's static wallpaper scope.


## Selected raster decoder dependency — 24 November 2026

- **Crate:** image 0.25.10.
- **Status:** selected in aurea-media Cargo.toml; Cargo.lock is intentionally not regenerated in this work session because the user requested no build/check/test cycle. The lockfile must be refreshed on Fedora before a locked build can succeed.
- **Enabled decoder features only:** bmp, gif, png, tiff, webp. default-features = false; Rayon, AVIF, EXR, DDS, HDR and other unrelated format features are not enabled.
- **Purpose:** safe Rust decoding adapter for BMP, GIF, PNG, TIFF and WebP. JPEG remains on the existing TurboJPEG adapter; no duplicate JPEG backend is enabled.
- **Upstream evidence:** image 0.25.10 documents these formats and recommends disabling default features for libraries. Its Limits supports strict maximum width/height; max allocation is best-effort and some decoders may ignore it.
- **License:** crate metadata/license expression and the enabled transitive dependency license set still require verification from the resolved Cargo.lock before distribution approval.
- **Debian 13:** Rust crate dependency; no system codec package required by this selection. Actual Trixie build and runtime behavior not yet tested.
- **Security note:** AureaOS applies axis limits before decode and independently validates pixel count/output bytes after decode. Because the crate documents max allocation as non-strict, this must not be treated as the sole memory guard. Worker queue remains bounded.
- **Pending:** regenerate Cargo.lock on Fedora, inspect the resolved graph/licenses, add format fixtures and decode/limit tests, then run one planned build/test cycle.

## Wallpaper asset ownership boundary — 24 November 2026

- **Status:** source implementation added; build verification pending.
- **Filesystem:** no new dependency. Standard Rust file I/O is confined to WallpaperAsset::from_file, which performs a metadata size check and bounded read before taking ownership of compressed bytes.
- **Threading contract:** callers must invoke file loading from settings/media-I/O work, not the compositor frame/render callback. Decoder execution remains on DecodeWorker.
- **Identity:** wallpaper asset IDs and texture/decode keys are owned strings; the old static identity requirement has been removed from this path.
- **Input detection:** JPEG, PNG, BMP, GIF, TIFF and WebP are identified by signatures. Extensions are not trusted for format selection. HEIC/HEIF and AVIF are not yet enabled.
- **Memory:** compressed input remains capped by DecodeLimits::default().max_input_bytes() (64 MiB). Owned compressed bytes are shared immutably within the asset, then copied into the bounded worker request. This temporary copy is intentional but must be considered in memory budgeting.
- **Verification:** no build/test/rustfmt performed. Review ownership API and snapshot call sites during the next planned Fedora validation.

## HEIF/AVIF backend verification update — 24 November 2026

Debian Trixie package metadata was rechecked:

- libheif source/runtime/development packages are available at 1.23.6-2~deb13u1 in the current security-updated Trixie metadata.
- HEVC still-image decode is supplied by libheif-plugin-libde265, which depends on libde265-0.
- AV1 still-image decode is supplied by libheif-plugin-dav1d, which depends on libdav1d7. The plugin's installed size is small (71 KiB amd64 in the current package listing); the codec runtime is additional.
- libheif documents HEIF as a container supporting HEVC- or AV1-coded images and loads codec implementations through separate plugins.
- Do not install the libheif-plugins-all metapackage. Select only the HEVC and AV1 decoder plugins if/when approved.
- Exact upstream license compatibility for libheif, libde265 and dav1d, ABI details, Fedora package parity and runtime plugin discovery remain to be reviewed. No native dependency has been added to Cargo.toml or the target package set.

Evidence: Debian Trixie package pages for libheif-dev, libheif-plugin-libde265 and libheif-plugin-dav1d; https://packages.debian.org/trixie/libheif-dev, https://packages.debian.org/trixie/libheif-plugin-libde265, https://packages.debian.org/trixie/libheif-plugin-dav1d.

## HEIF/AVIF native dependency CI gate — 24 November 2026

- **CI environment:** dedicated debian:trixie container job, separate from the Ubuntu 24.04 workspace build.
- **Build-time packages under verification:** libheif-dev and pkg-config.
- **Decoder-only plugin packages under verification:** libheif-plugin-libde265 (HEVC still images) and libheif-plugin-dav1d (AV1 still images).
- **API baseline:** CI requires pkg-config --atleast-version=1.20 libheif, because the planned safe wrapper integration needs the context memory-limit and sequence-detection APIs.
- **Scope:** package installation, pkg-config version and plugin shared-library presence only. This job does not yet compile libheif-rs, link Aurea Media, or approve production adoption.
- **Security/maintenance:** the upstream parser-risk review remains a gate. Keep native decoding isolated behind aurea-media; no native codec code may enter the compositor/render thread.
- **Status:** CI verification pending. No Cargo dependency, lockfile, runtime image manifest or Fedora package decision has been added.


## Noto Sans — desktop context-menu typography — 25 November 2026

- **Selection:** Noto Sans from Google Fonts / The Noto Project, Regular 500 weight for the initial desktop context menu.
- **License:** SIL Open Font License 1.1 (OFL-1.1). The font family may be used and redistributed with software under the OFL conditions; preserve the upstream copyright and license notice if font files are ever bundled. See the [Noto usage and license guidance](https://notofonts.github.io/noto-docs/website/use/).
- **Build-time use:** SVG masters declare font-family=Noto Sans; the host rsvg-convert rasterizes them into RGBA8. The font file is not linked by Rust and is not required at runtime once the raster assets are packaged.
- **Host packages:** Debian 13 fonts-noto-core; Fedora google-noto-sans-fonts. Existing host rasterizer packages remain Debian librsvg2-bin and Fedora librsvg2-tools.
- **Runtime impact:** no Cargo dependency, no fontconfig/Pango/Cairo/GLib dependency added to AureaOS runtime. No external font download or network access occurs at runtime.
- **Status:** SVG source masters and the deterministic export script are committed. Raster generation and visual inspection are pending on Fedora.
