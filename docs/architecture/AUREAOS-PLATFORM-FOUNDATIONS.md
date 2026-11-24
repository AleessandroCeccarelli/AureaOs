<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Platform Foundations

## Status
Forward-looking architecture baseline. This document records product direction; it does not claim that the described capabilities are implemented.

## Product vision
AureaOS is a coherent Linux-native desktop platform built on Debian, Btrfs, Rust, Wayland, Smithay and Vulkan. It should provide the continuity and usability expected from mature desktop systems without copying their interfaces, private services or implementations.

Guiding question: **what experience should the user have, and what is the correct Linux-native way to deliver it?** Users should not need Linux expertise to use the desktop, while AureaOS respects Linux filesystem semantics, permissions, interoperability and standards.

## Platform baseline
- Debian GNU/Linux 13 Trixie, amd64 initially.
- Btrfs as the selected system filesystem, with one Btrfs partition for the complete installed OS; Aurea Time owns backup and restore.
- systemd-boot as the UEFI-only boot manager; Secure Boot is disabled for Roma 1.0.
- Rust as the primary language for services, APIs, shell and desktop components.
- Wayland + Smithay as the native display/compositor architecture; no X11 or XWayland.
- Vulkan as the exclusive graphics API; no OpenGL rendering fallback.
- FreeDesktop/XDG conventions for interoperability.
- GPL-3.0-or-later.

These choices are constraints, not a reason to duplicate facilities already provided by Linux.

## Boot architecture
The approved boot manager is systemd-boot. AureaOS targets UEFI systems only; Legacy BIOS/CSM is unsupported. Secure Boot is disabled for Roma 1.0 because Microsoft signing/certification costs are not a sustainable first-release prerequisite. UKI is the selected boot-artifact direction, subject to validation of Debian 13 kernel, initramfs-tools and systemd integration. The ESP stores systemd-boot and UKIs; the installed OS remains on one Btrfs partition. No XBOOTLDR partition is planned. See [systemd-boot and UKI architecture](SYSTEMD-BOOT-AND-UKI.md).

## System boundaries
- **Aurea Desktop:** desktop surface, item placement, selection and presentation state.
- **Fides:** file navigation and file operations.
- **Quaero:** universal search and conversational system entry point.
- **Aurea Intelligence:** orchestration, local context policy and provider boundaries.
- **Aurea File Intelligence:** local file catalog, metadata, indexing and permission-aware queries.
- **Aurea Media:** bounded media inspection, decoding and thumbnail generation.
- **Aurea Passkey:** system authentication and credential integration.
- **Aurea Compositor:** Wayland surfaces, outputs, input routing and Vulkan composition only.

Keep APIs small and explicit. The desktop composition root may host session-scoped services initially. A permanent daemon or privileged service needs a concrete lifecycle, isolation or multi-client justification.

## Quaero and Aurea Intelligence
Quaero is the system-wide search and assistant experience. Its direction is a ChatGPT-like conversational interface available at OS level and integrated with Aurea Intelligence, not merely a search box.

It should understand natural-language requests involving authorized system capabilities: applications, files, settings, help and eventually permitted actions. Results and actions retain provenance and permission checks.

### Local-first provider model
- Local search, application discovery, settings lookup and essential desktop operations work offline.
- Aurea Intelligence owns orchestration, context policy, provider selection and capability boundaries.
- ChatGPT can be an explicitly configured remote provider, never a hidden dependency for boot, login, basic search or desktop operation.
- Remote requests disclose what context is sent and require appropriate authorization.
- Private file contents, credentials, tokens and unrelated personal data are never silently uploaded.
- Remote output is untrusted and cannot bypass authorization, filesystem permissions, confirmations or command policy.
- Destructive, financial, security-sensitive or externally visible actions require explicit confirmation and a clear preview.
- The system remains usable when network or remote provider is unavailable.

Quaero owns UX and intent presentation; Aurea Intelligence owns orchestration and policy; File Intelligence and other services own their data. The assistant must not become a second filesystem index or unrestricted shell.

## Aurea Passkey: system authentication
Aurea Passkey is intended as a consistent system-level credential and authentication capability, not merely a password manager.

Target integration points:
- user login and session unlock;
- privilege elevation and authorization prompts;
- system settings and sensitive operations;
- applications through documented, consent-based APIs;
- external FIDO2 security keys and compatible authenticators;
- TPM 2.0-backed protection when supported by hardware and firmware.

### Security architecture
Separate protocol handling, credential storage, policy and UI:
1. **Authentication broker:** session/system entry point mediating requests and enforcing policy.
2. **Authenticator adapters:** FIDO2/CTAP transports and platform authenticator integration.
3. **Credential store:** protected public credential material and metadata; private keys remain with the authenticator whenever the protocol allows.
4. **Hardware trust adapter:** optional TPM 2.0 integration to seal/protect local secrets. TPM does not automatically mean every passkey is stored inside it.
5. **PAM/session integration:** scoped adapter for login/unlock and privilege workflows, designed with recovery and fallback.
6. **User experience:** enrollment, authenticator selection, consent, recovery and auditable status.

WebAuthn and CTAP2 are related but distinct: WebAuthn defines the relying-party/client ceremony; CTAP connects a client to an authenticator. A local Linux login is not automatically a WebAuthn relying-party flow. Reuse protocol libraries only after checking exact scope, licensing, MSRV, transports and security posture.

A passkey must not be the sole recovery path in the first release. Preserve secure recovery, account administration and documented fallback. Do not invent proprietary cryptography or store raw private keys in ordinary application configuration. TPM support is optional and capability-detected. The compositor never owns credential secrets.

## Btrfs system filesystem and Aurea Time
Btrfs is selected as AureaOS's modern Linux filesystem. Its selection does not imply a plan to use native Btrfs snapshots, snapshot-based updates or rollback.

The installed disk layout is intentionally simple:
- One EFI System Partition (ESP) for UEFI boot artifacts.
- One Btrfs partition containing the entire installed operating system.
- No separate partitions for `/home`, `/var`, cache, snapshots or backups.
- No XBOOTLDR partition in the current boot design.

Aurea Time is the single planned AureaOS backup and restore system. It will target external USB storage or a NAS and provide time-based recovery from individual files and folders through settings and the complete operating system. Its implementation, consistency model, storage format, retention and full-system recovery workflow remain to be designed.

An independent Aurea Recovery environment is required so the operating system can be restored even when the installed system does not boot. Recovery must not depend on Btrfs snapshots.

Btrfs feature tuning—including subvolumes, compression, scrub, quotas, RAID, encryption and swap/hibernation—has not been selected as part of this baseline. Each feature requires a separate technical justification and explicit decision.

## File Intelligence and local data
Aurea File Intelligence remains the shared local catalog for Fides, Quaero and authorized clients. It must be permission-aware, incremental, resource-bounded and rebuildable.

SQLite remains the leading embedded database candidate, not an approved runtime dependency until FTS5, Rust bindings, MSRV, system linking, packaging and recovery are validated on Debian and Fedora. User-authored metadata remains separate from rebuildable index data.

Initial search prioritizes filename, path, MIME, inexpensive metadata and explicitly indexed text. Semantic/vector search and model-based extraction are later optional layers, not prerequisites for basic search.

## Resource, privacy and reliability
- Keep rendering/input paths free of indexing, decoding, AI calls and blocking storage work.
- Use bounded workers, cancellation, backpressure and observable resource budgets.
- Prefer local processing; make remote processing explicit.
- Respect filesystem permissions at every query/action boundary.
- Treat files, metadata, model output and external devices as untrusted input.
- Expose offline, stale and degraded states honestly.
- Keep APIs small, independent of UI frameworks and versioned when necessary.
- Avoid giant modules: one file, one responsibility.

## Technology evaluation notes
- Debian 13 provides SQLite runtime and development packages. Fedora package metadata reports FTS5 enabled. Debian package listings alone do not prove the installed library's compile options; verify FTS5 on the target image.
- Current notify 9 documentation reports Linux inotify support, CC0 licensing and MSRV 1.91. The inspected workspace uses Rust edition 2024 but declares no rust-version. Do not adopt notify 9 until the toolchain/MSRV policy is explicit; a narrow native inotify adapter remains an alternative.
- Rust passkey libraries have different scopes. The 1Password passkey-rs family covers WebAuthn client and CTAP2 authenticator components; Linux FIDO libraries have distinct transport and licensing constraints. No crate is approved here.
- Native Btrfs snapshots are not part of the planned AureaOS backup, update or restore workflow.

## Delivery sequence
1. Freeze platform principles and ownership boundaries.
2. Define Aurea Time backup/restore contracts and the independent Aurea Recovery environment.
3. Define Aurea Passkey threat model, broker contract, recovery and PAM strategy.
4. Define Aurea Intelligence provider, privacy and action-authorization contracts.
5. Complete File Intelligence database/watcher compatibility evaluation.
6. Implement small, independently testable service modules and integrate incrementally.
7. Validate on Debian 13 and Fedora hardware, including constrained machines and offline operation.

This document does not imply implementation, build, test or runtime availability.
