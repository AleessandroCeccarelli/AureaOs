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

### JPEG decoding — TurboJPEG / libjpeg-turbo

- **Status:** Native library is available on Debian; the currently reviewed Rust binding path is rejected for Debian 13 integration. Direct FFI remains unapproved.
- **Purpose:** Decode the initial JPEG wallpaper without a general-purpose image stack.
- **Debian evidence verified:** Debian 13 Trixie amd64 provides `libturbojpeg0` and `libturbojpeg0-dev`, both version `1:2.1.5-4`. The development package depends on the exact matching runtime package.
- **Fedora development evidence:** Fedora publishes `libjpeg-turbo-devel` and `turbojpeg-devel`; current package pages list Fedora 43/44/45 package streams with libjpeg-turbo 3.1.x. This version difference is material and must not be hidden by testing only on Fedora.
- **Rust binding finding:** `turbojpeg-sys` 1.1.1 and 1.2.0 expose TurboJPEG 3.0's `tj3*` API; the high-level `turbojpeg` crate 1.5.1 depends on that binding family. They cannot be assumed to link against Debian Trixie's TurboJPEG 2.1 ABI/API.
- **Decision:** Do not add `turbojpeg` or `turbojpeg-sys` to AureaOS for the Debian 13 system library path. Reconsider only if a verified TurboJPEG 2.1 binding or a deliberately maintained Aurea-owned FFI adapter is justified and tested against Debian's headers/library.
- **Alternative candidate:** `zune-jpeg` 0.5.15 is a pure-Rust decoder with one declared normal dependency, `zune-core ^0.5.1`; its manifest declares MSRV 1.75 and `MIT OR Apache-2.0 OR Zlib`. This avoids a native codec ABI dependency, but it remains **Candidate**, not Approved: inspect its complete transitive graph, maintenance/security status, decoder limits and failure behavior before integration.
- **Architecture:** Whichever decoder is selected must sit behind a small Aurea-owned adapter and run on a bounded worker, never in the render loop. Enforce input byte, dimension, output allocation and total memory limits.

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
