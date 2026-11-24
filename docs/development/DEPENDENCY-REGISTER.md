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

- **Status:** Candidate — a focused native-library adapter is the leading option, but it is not approved for implementation until the ABI/symbol smoke test and decoder-boundary review pass on both development and target distributions.
- **Purpose:** Decode the initial JPEG wallpaper into bounded RGBA8 pixels without introducing a general-purpose image stack.
- **Debian evidence verified:** Debian 13 Trixie amd64 provides `libturbojpeg0` and `libturbojpeg0-dev`, version `1:2.1.5-4`. The development package depends on the matching runtime package.
- **Fedora development evidence:** Fedora provides `libjpeg-turbo-devel` and `turbojpeg-devel`; the package streams reviewed use libjpeg-turbo 3.1.x. The exact installed Fedora package build remains to be checked on the user's machine.
- **Compatibility finding:** Upstream libjpeg-turbo 3.x release notes explicitly describe a TurboJPEG 2.x compatibility wrapper. This makes a small adapter around the legacy API a plausible bridge between Debian's 2.1 package and Fedora's 3.1 package, but the exact exported symbols, declarations and runtime linking must be verified against both distributions' actual headers/libraries before approval.
- **Rust binding decision:** Do not use the currently reviewed `turbojpeg` / `turbojpeg-sys` crate path: the reviewed bindings expose the TurboJPEG 3.0 `tj3*` API and cannot be assumed compatible with Debian's 2.1 system library. Do not add bindgen or a broad image crate for this one decoder.
- **Proposed integration boundary:** A small Aurea-owned Rust adapter with all unsafe FFI isolated in one module; expose only a safe decode contract returning the existing bounded `WallpaperPixels`. Decode on a bounded worker, never on the render thread. Validate compressed input length, decoded dimensions, checked RGBA allocation, maximum pixel count and total memory budget before allocating.
- **Alternatives reviewed:** `zune-jpeg` 0.5.15 is blocked because upstream reports incorrect colors for valid baseline JPEGs; the identified fix is only in 0.5.16-rc3, a pre-release. `jpeg-decoder` 0.3.2 is pure Rust but upstream describes it as maintenance-mode; Debian Trixie packages 0.3.0. Neither is currently selected as the primary path.
- **Open approval checks:** Verify exact legacy API symbols and ABI on Debian 13 and Fedora; confirm the runtime package is included in the target image; review current libjpeg-turbo security advisories and BSD-style license notices; define panic/error containment and worker cancellation; test baseline/progressive, grayscale, truncated and malformed inputs plus the committed wallpaper.
- **Validation:** No native library was installed or linked in this repository change. No Cargo manifest or lockfile was modified. No build, format, check or tests were run.
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

