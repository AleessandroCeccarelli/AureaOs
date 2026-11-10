<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Application and Component Catalog

This document is the authoritative inventory of named first-party applications, desktop components, system tools, services, shared design resources, and selected third-party application categories in AureaOS.

It records product direction, technical foundations, and implementation status. Inclusion in this catalog does not mean that a component is complete or available to users.

## Platform and Status

AureaOS Roma 1.0 targets Debian GNU/Linux 13 (Trixie), `amd64` (x86-64).

AureaOS is GUI-first and Wayland-first. The repository contains an early Debian root filesystem bootstrap, package manifests, visual design resources, and initial Rust implementations of the desktop session coordinator and Wayland compositor.

The desktop foundation is under active development. It is not yet a complete graphical desktop or a bootable operating system. No complete AureaOS desktop UI or first-party end-user application is available.

### Status Definitions

- **Implemented:** the described functionality exists in the repository.
- **Implemented prototype:** an initial working implementation exists, but the component is incomplete and not ready as a finished product.
- **Implemented foundation:** documentation, design resources, specifications, or other groundwork exists; the complete component is not implemented.
- **Planned:** included in the current product direction. Roma 1.0 priority reflects current product direction, not a delivery commitment.
- **Candidate:** documented concept or possible future component whose scope or continued use is not confirmed.
- **Undecided:** no settled product or technical choice is documented.

Rust is the primary programming language for AureaOS-developed applications and components where appropriate. A non-Rust choice requires a documented technical reason. This is not a requirement for third-party software, visual assets, or web styling such as CSS.

Vulkan is the selected and exclusive graphics rendering API for AureaOS. Its complete rendering and presentation pipeline is not yet implemented.

No native UI toolkit has been selected.

## Catalog

| Name | Category | Name origin | Purpose | Status | Programming language | Technology / toolkit | UI integration |
|---|---|---|---|---|---|---|---|
| **Aurea Desktop** | Desktop/UI component | Project name; a specific etymology is not recorded. | Complete desktop experience, including shell, window management, system interface, and everyday desktop interactions. | Implemented foundation: initial shell and compositor components exist. Complete desktop UI and interactions remain planned; Roma 1.0 direction. | Rust primary; justified exceptions may be considered. | Wayland-first. Smithay is used by the current compositor prototype. Vulkan is the selected exclusive rendering API. Native UI toolkit remains undecided. | Shared AureaOS Human Interfaces contract; light/dark themes, typography, keyboard and accessibility support. |
| **Aurea Shell (`aurea-shell`)** | Desktop/session component | Descriptive project name. | Coordinate desktop session lifecycle, start and coordinate the compositor, and handle session-level events and shutdown. | Implemented prototype. Session lifecycle coordination, event bus, compositor startup, and shutdown handling are present. Not a complete graphical shell. | Rust. | Separate Rust workspace component; event-driven lifecycle coordination. | No complete graphical shell UI yet. Future interface must follow AureaOS Human Interfaces. |
| **Aurea Compositor (`aurea-compositor`)** | Desktop/Wayland component | Descriptive project name. | Provide the Wayland server, surface and output lifecycle, and compositor responsibilities. | Implemented prototype. Initial Wayland server, XDG shell integration, toplevel tracking, DRM device lifecycle, output management, and Wayland output publication are present. Rendering, physical presentation, complete input handling, and complete window management remain incomplete. | Rust. | Smithay 0.7, Calloop, libseat, Udev, Linux DRM interfaces, and Vulkan as the selected rendering API. | Wayland protocol foundation. No finished desktop interface. |
| **AureaOS Human Interfaces** (also called “Aurea Human Interfaces” in an earlier roadmap) | Library | Descriptive project name; no separate etymology is recorded. | Shared design system and, in future, reusable UI components for first-party applications. | Implemented foundation: design specifications, CSS theme tokens, and visual resources exist. Shared native component library is planned, not implemented. | Rust primary for a future native library; CSS is used for existing web theme tokens. | `aurea-human-interfaces/design-system/tokens/index.css` is the source of truth for design tokens. Light and dark CSS themes are provided. Native toolkit and component-library implementation remain undecided. | Defines the shared contract in [Human Interfaces](../human-interfaces/README.md). |
| **AureaOS System API** (earlier wording: “Aurea System API”) | Library | Descriptive project name; no separate etymology is recorded. | Shared interfaces for system capabilities used by desktop components and applications. | Planned; scope and interfaces undecided. | Rust primary where appropriate. | Service boundaries, protocols, and implementation are undecided. | Must expose accessible, permission-aware behavior to UI clients and align with shared terminology and interaction patterns. |
| **Quaero** | System application | Latin *quaerō*, “I seek” or “I search,” from *quaerere* ([Lewis & Short via Logeion](https://logeion.org/quaero)). | Global search for applications, files, folders, settings, and supported system actions. | Planned; Roma 1.0 direction. | Rust primary; justified exceptions may be considered. | Search/indexing backend and UI toolkit are undecided. | Shared design tokens, light/dark themes, typography direction, keyboard-first navigation, visible focus, and accessible results. |
| **Fides** | System application | Latin *fidēs*, “faith,” “trust,” or “confidence” ([Lewis & Short via Scaife ATLAS](https://atlas.perseus.tufts.edu/dictionaries/entry/urn%3Acite2%3Ascaife-viewer%3Adictionary-entries.atlas_v1%3Alat.ls.perseus-eng2-n18127/)). | File manager for common local and supported remote file workflows. | Planned; Roma 1.0 direction. | Rust primary; justified exceptions may be considered. | File, archive, and preview infrastructure and UI toolkit are undecided. | Shared design tokens and themes; accessible file navigation, keyboard operation, focus states, and plain-language errors. |
| **Aurea Wallet** | System application and security-critical component | Project name; a specific etymology is not recorded. | First-party credential and payment-card manager. Stores and manages passwords, passkeys, and payment-card data through a dedicated encrypted credential vault. It is a mandatory component of AureaOS Roma 1.0. | Planned; mandatory Roma 1.0 release requirement. AureaOS Roma 1.0 will not be released without a functioning Aurea Wallet that has passed security validation. | Rust primary; security-driven exceptions require documented justification. | Dedicated Aurea Wallet Security Service, Aurea Secure Vault, encrypted credential storage, TPM 2.0, UEFI Secure Boot, and platform-integrity verification. Detailed architecture is documented in [Aurea Wallet Architecture](../architecture/AUREA-WALLET.md). | Must follow shared UI patterns and accessible consent/security messages. The UI must remain separate from the security service and must never directly control the credential vault or its master keys. |
| **Aurea Store** | System application | Project name; a specific etymology is not recorded. | Official AureaOS graphical application store for discovering, installing, launching, removing, and updating third-party applications. | Planned; Roma 1.0 direction. | Rust primary for AureaOS code; justified exceptions may be considered. | Integrates with Flathub as an application source and Flatpak for installation, execution, sandboxing, and updates. Store backend, toolkit, and trust/provenance presentation remain to be designed. | Shared themes, typography, terminology, keyboard navigation, permission explanations, and accessible status feedback. |
| **Aurea Update** | System application | Project name; a specific etymology is not recorded. | Graphical updates for AureaOS-specific system components. | Planned; Roma 1.0 direction. | Rust primary; justified exceptions may be considered. | Must use an update channel separate from the Debian base. Mechanism, signing, compatibility, rollback, and toolkit are undecided. Flatpak application updates are a separate domain. | Shared themes and components; accessible progress, errors, restart notices, and plain-language explanations. |
| **Aurea Time** | System application | Project name; a specific etymology is not recorded. | System backup and restore, from an individual file through the complete system, using compatible USB disks or network storage. | Planned; name and role confirmed, Roma 1.0 direction. | Rust primary; justified exceptions may be considered. | Btrfs is the planned default filesystem, without using its native snapshot workflow for backup or restore. Aurea Time owns those tasks; its backup format, scheduling, storage compatibility, integrity checks, and toolkit remain open. | Shared themes and typography; accessible backup/restore setup, clear destination and progress information, and plain-language recovery messages. |
| **Aurea Sans** | Library | Project typeface name; a specific linguistic origin is not recorded. | Intended primary typeface and typography identity for AureaOS interfaces. | Planned. Font files in the repository are source assets; the selected Aurea Sans family and final fallback stack are not confirmed. | Not applicable; typeface, not software. | Typeface selection, licensing, and fallback fonts are open decisions. | Primary typography direction across first-party UI; provide readable fallbacks, scalable text, and legible light/dark use. |
| **Sonus** | System application | Existing application note gives Latin *sonus*, “sound.” | Candidate music player for local audio collections. | Candidate; Roma 1.0 inclusion is not confirmed. | Rust primary if selected; justified exceptions may be considered. | Audio backend, codecs, and UI toolkit are undecided. | If selected, use shared tokens/themes, typography direction, accessible playback controls, keyboard support, and consistent media terminology. |
| **Visio** | System application | Existing application note gives Latin *visiō*, “sight” or “vision.” | Candidate video player for local media. | Candidate; Roma 1.0 inclusion is not confirmed. | Rust primary if selected; justified exceptions may be considered. | Playback infrastructure, codec support, and UI toolkit are undecided. | If selected, follow shared themes and accessibility rules; controls may suit video viewing while retaining AureaOS visual identity. |
| **Explora** | System application | Existing note gives Latin *explōrā* (“explore!”), from *explōrāre*; spelling and grammatical gloss should be verified. | Possible future AureaOS first-party web browser for everyday navigation. | Candidate; not part of the Roma 1.0 first-release application commitment. | Rust primary for AureaOS code if selected; browser engine may be third-party. | Browser engine/upstream and UI toolkit are undecided; evaluate existing engines and licensing. | If selected in a future release, use shared AureaOS chrome where practical, with keyboard/accessibility support and clear browser-specific terminology. |
| **Aurea Greeter** | Desktop/UI component | Project name; a specific etymology is not recorded. | Earlier roadmap name for graphical login and session entry. | Candidate from an earlier roadmap; current scope and name are unconfirmed. | Rust primary if implemented as AureaOS software. | Display manager/session integration and toolkit are undecided. | If retained, align with shared typography, themes, keyboard access, accessibility, and plain-language authentication messages. |
| **Printer management** | System tool | Functional name; not a branded product name. | Graphical setup, configuration, status, and queue management for supported printers. | Planned for Roma 1.0; no implementation. | Rust primary for AureaOS interface/tooling; justified exceptions may be considered. | CUPS and suitable Linux drivers are proposals to evaluate, not confirmed integration. | Integrate with system settings and shared UI patterns; expose only supported options and provide accessible status/errors. |
| **Scanner management** | System tool | Functional name; not a branded product name. | Discover supported scanners and provide a graphical scanning workflow. | Planned for Roma 1.0; no implementation. | Rust primary for AureaOS interface/tooling; justified exceptions may be considered. | SANE and compatible backends are proposals to evaluate; toolkit and scan application boundary are undecided. | Shared themes and controls; keyboard-accessible device selection, preview, progress, and errors. |
| **Resource Monitor** | System tool | Functional name; not a branded product name. | Show available system-resource activity and processes. | Planned for Roma 1.0; no implementation. | Rust primary; justified exceptions may be considered. | Telemetry sources, sampling, and UI toolkit are undecided; hardware metrics must be capability-dependent. | Shared themes, typography, accessible graphs/labels, keyboard navigation, and clear units. |
| **System-level passkeys** | Service | Descriptive capability name. | Explore consistent passkey use by compatible applications and services. | Candidate, exploratory/long-term; scope and support are undecided. | Rust primary for AureaOS components where appropriate. | Standards integration, storage, permissions, and recovery design are open decisions. | Shared accessible authentication/consent patterns and clear user-facing terminology; no implementation exists. |
| **Aurea AI / ChatGPT integration** | Service | “Aurea AI” is a project label; ChatGPT is the name of the external service. No etymology is claimed. | Explore system-level access to AI assistance, subject to service/API, privacy, and permission constraints. | Candidate, exploratory/long-term; not a Roma 1.0 commitment. | Rust primary for AureaOS integration where appropriate; external service implementation is outside this choice. | API, online/service dependencies, permissions, privacy model, and UI are undecided. | Shared UI and language contract; disclose external data sharing, support keyboard/accessibility, and do not imply unrestricted local access. |
| **Aurea Media API** | Library | Project name; a specific etymology is not recorded. | Earlier roadmap concept for shared multimedia and format handling across first-party applications. | Candidate from an earlier roadmap; current scope and Roma 1.0 priority are unconfirmed. | Rust primary if pursued; justified exceptions may be considered. | Codec/library choices, API, and licensing remain open and require review. | Shared interface contract for any user-facing controls; service APIs should report supported formats/capabilities consistently. |

## Third-Party Applications

AureaOS Roma 1.0 will provide selected third-party applications for common desktop tasks that are not priorities for first-party development.

The initial product direction includes:

- Web browsers.
- Email clients.
- Office and productivity suites.
- Other established desktop applications useful for everyday work.

These applications are not AureaOS first-party products and do not need to be written in Rust.

Aurea Store will integrate with Flathub and Flatpak to make compatible third-party applications discoverable, installable, runnable, and updatable.

Third-party Flatpak applications run in their respective sandbox environments. Their permissions, provenance, maintenance, licensing, and security must be presented and evaluated appropriately.

The use of third-party applications allows AureaOS to concentrate its development effort on its own desktop environment, system integration, Wallet, file manager, global search, store, and other core system components.

A future AureaOS release may consider first-party replacements for selected application categories when resources and technical priorities make that practical.

## Shared UI and Language Contract

All first-party desktop and system applications should follow [AureaOS Human Interfaces](../human-interfaces/README.md), including shared semantic design tokens, light and dark themes, typography, spacing, controls, icons, and interaction patterns.

The current CSS token set in `aurea-human-interfaces/design-system/tokens/` is the source of truth for implemented design values. Native toolkit mappings should consume the same semantic contract rather than create competing palettes.

Reusable cross-toolkit UI components are not implemented.

Aurea Sans remains the primary typeface candidate, not a confirmed font selection. The final font and fallback stack remain open.

Interfaces must support:

- Keyboard navigation.
- Visible focus.
- Accessible labels and feedback.
- Scalable text.
- Reduced-motion preferences.
- Consistent product names and plain-language copy.

Application layouts may differ to suit their tasks while retaining AureaOS visual and linguistic identity.

The default UI language and localization policy have not been established and remain open decisions.

## Scope Notes

The current Roma 1.0 direction includes:

- Aurea Desktop and its desktop/session foundation.
- AureaOS Human Interfaces and shared design resources.
- Quaero global search.
- Fides file manager.
- Aurea Wallet as a mandatory first-party system application and release requirement.
- Aurea Store and its Flatpak/Flathub integration.
- Aurea Update.
- Aurea Time.
- The system tools labeled “planned for Roma 1.0” in the catalog.

AureaOS Roma 1.0 will not be released without a functioning Aurea Wallet that has passed security validation. Wallet is a release requirement, not an optional application or a feature that may be postponed to a later release.

Their scope and implementation remain subject to development and validation. Inclusion in the Roma 1.0 direction is not a claim that the components are already implemented or complete.

The following remain candidates or exploratory ideas, not Roma 1.0 commitments:

- Sonus.
- Visio.
- Explora as a first-party browser.
- Aurea Greeter.
- Aurea Media API.
- System-level passkeys.
- Aurea AI / ChatGPT integration.

Third-party browsers, email clients, office suites, and other selected applications are intended to cover common application needs in Roma 1.0. They are separate from the AureaOS first-party application catalog.

Aurea Time is the single planned system for backup and restore, including full-system restore. Btrfs native snapshots are not part of the design.

Update domains remain distinct:

1. Debian base packages.
2. AureaOS-specific system components.
3. Flatpak applications.

The mechanisms for the first two update channels are not yet implemented.

## Display Intelligence

| Name | Category | Name origin | Purpose | Status | Programming language | Technology / toolkit | UI integration |
|---|---|---|---|---|---|---|---|
| **Aurea Display Intelligence** | System subsystem | Descriptive project name. | Discover display capabilities and coordinate display mode and refresh policies. | Planned; architecture documented in [Display Intelligence](../architecture/DISPLAY-INTELLIGENCE.md). Not implemented. | Rust primary where appropriate. | DRM/KMS backend, compositor integration, and service boundary are planned; implementation details remain open. | Exposed through Settings → Display & Video, with automatic and manual modes. |
| **Display Capability Service** | System service | Functional name. | Discover and maintain per-display capabilities, including supported modes, connector identity, and VRR availability and limits where reliably exposed. | Planned; part of Aurea Display Intelligence. Initial connector and mode discovery foundations exist in the compositor. | Rust primary where appropriate. | DRM/KMS, connector properties, EDID, and driver-reported capabilities; exact interfaces remain open. | Supplies capability information to display settings and policy; not a separate user-facing application. |
| **Aurea VRR** | System service / display strategy | Project name plus standard abbreviation for Variable Refresh Rate. | Use variable refresh on compatible display pipelines, coordinating with frame presentation within effective hardware and driver limits. | Planned; part of Aurea Display Intelligence. | Rust primary where appropriate. | DRM/KMS VRR properties and compositor presentation timing; availability depends on the complete GPU, driver, connector, and display path. | Automatic management when supported; show VRR availability and effective status in Display & Video settings. |
| **Aurea Fixed Refresh** | System service / display strategy | Descriptive project name. | Select a suitable supported fixed refresh mode when VRR is unavailable or unsuitable. | Planned; part of Aurea Display Intelligence. | Rust primary where appropriate. | DRM/KMS mode setting through the selected graphics driver; policy and safe fallback behavior are to be implemented. | Automatic fallback and manual fixed-rate selection in Display & Video settings. |
| **Display Policy Engine** | System service | Functional name. | Choose per-display resolution and refresh policy using hardware capabilities, frame cadence, desktop context, power policy, and user preferences. | Planned; part of Aurea Display Intelligence. | Rust primary where appropriate. | Policy module integrated with display capability discovery and the DRM/KMS backend; exact service API remains open. | Provides the Automatic — AureaOS behavior and respects manual overrides. |

These display components are architectural modules within Aurea Display Intelligence, not five separate applications or independent daemons by default.

Their final process boundaries will be decided during implementation.

The compositor supplies output lifecycle and presentation information and coordinates changes, but it must not become the owner of the entire display policy.

DRM/KMS and the graphics driver expose and apply hardware capabilities; AureaOS owns the policy and user experience.

No universal VRR range or minimum refresh rate is assumed.
