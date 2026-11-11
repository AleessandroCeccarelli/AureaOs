<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Roadmap

This roadmap records the current implementation status and intended development priorities. It is a working plan, not a delivery schedule or a promise of release dates.

AureaOS Roma 1.0 targets Debian GNU/Linux 13 (Trixie), `amd64` (x86-64).

AureaOS uses Debian as its lower-level system foundation. The desktop environment, system experience, core applications, and product identity are developed and maintained by the AureaOS project.

## Product Direction

The first release focuses on delivering a complete and coherent AureaOS desktop experience without attempting to develop every application from scratch.

The Roma 1.0 product direction includes:

- A proprietary AureaOS graphical desktop and Wayland session.
- AureaOS Human Interfaces and a consistent visual design system.
- Aurea Wallet as a first-party system application and a core product pillar.
- Fides, the AureaOS file manager.
- Quaero, the global search experience.
- Aurea Store, the official AureaOS application store.
- Aurea Update for AureaOS system components.
- Aurea Time for system backup and restore.
- Essential system tools and graphical configuration workflows.

Common application categories such as web browsers, email clients, and office suites will initially be covered by established third-party applications.

Aurea Store will integrate with Flathub and Flatpak to discover, install, run, remove, and update compatible third-party applications in their sandbox environments.

Developing first-party replacements for these application categories may be considered in future releases. It is not a requirement for Roma 1.0.

## Current Status

### Implemented in the Repository

- Project vision, contribution guidance, technical direction, and initial architecture documentation.
- Debian package manifests and a bootstrap script for a minimal, terminal-only Debian root filesystem.
- AureaOS visual design resources, including design tokens, themes, icons, illustrations, and wallpapers.
- A four-crate Rust workspace under `components/`: `aurea-desktop`, `aurea-shell`, `aurea-desktop-api`, and `aurea-compositor`.
- An initial `aurea-desktop` composition root that wires the desktop components together; this is not a finished desktop session or graphical interface.
- Shared desktop contracts in `aurea-desktop-api`.
- Shell session lifecycle coordination and shutdown handling through an event bus.
- A Smithay-based Wayland server with initial XDG shell integration.
- Initial Wayland seat and XKB keyboard support, with libinput event routing under the DRM feature.
- Global Super + Ctrl + Left / Right workspace navigation shortcuts.
- Initial toplevel surface tracking and window-management structures.
- Initial four-workspace model with workspace switching and per-window assignment.
- Render-scene filtering so only mapped windows on the active workspace are drawn.
- Conversion of mapped XDG surface trees into Smithay render elements, with logical window positions and output scaling.
- DRM device discovery and lifecycle management through Udev and libseat.
- DRM device event integration with the compositor event loop.
- Session pause and activation handling for DRM devices.
- GPU inventory and rendering preference models.
- DRM connector discovery and supported display mode reporting.
- Stable output identity generation.
- Output registration, removal, configuration, and logical layout management.
- Output reconciliation when discovered display configurations change.
- Publication and removal of registered outputs as Wayland output globals.

These components establish an initial desktop and graphics-device foundation. They do not yet constitute a usable graphical desktop or a complete graphics pipeline.

The Debian bootstrap profile still needs package-resolution validation and does not produce a bootable installation image.

### In Development / Validation

- Validate the complete shell and compositor workspace through compilation and automated tests.
- Continue hardware-independent testing of output management and DRM lifecycle behavior.
- Validate DRM device acquisition, output discovery, hotplug, and session handling on representative physical hardware.
- Define and validate system identity, service defaults, storage layout, boot configuration, and security policy.
- Validate the Debian 13 package profile and bootstrap process against the intended repositories.

## Planned Development

### 1. Desktop and Graphics Foundation

- Implement the concrete Vulkan rendering backend and physical frame presentation.
- Complete DRM/KMS mode setting and physical display configuration.
- Complete pointer, touch, and gesture input handling, including touchpad workspace gestures.
- Develop complete window management and desktop interactions.
- Integrate compositor rendering, presentation, and output lifecycle responsibilities.
- Validate the graphics pipeline on representative hardware.

Vulkan is the selected and exclusive graphics rendering API for AureaOS. A complete rendering and presentation pipeline remains to be implemented.

### 2. Display Intelligence

- Develop Aurea Display Intelligence for display capability discovery and display policy.
- Implement fixed-refresh strategies and VRR support where the complete hardware and driver path allows it.
- Integrate display policy with compositor presentation information and the DRM/KMS backend.
- Provide Automatic — AureaOS and manual display configuration through Settings → Display & Video.
- Implement safe fallbacks for unsupported or unavailable display capabilities.

The architecture and responsibilities are documented in [Display Intelligence](docs/architecture/DISPLAY-INTELLIGENCE.md).

### 3. AureaOS Desktop Experience

- Develop the graphical desktop shell.
- Develop the shared AureaOS Human Interfaces library and reusable UI components.
- Implement the window-management experience and desktop interactions.
- Develop system settings and essential graphical configuration workflows.
- Integrate notifications, keyboard navigation, accessibility, themes, and localization foundations.
- Build the graphical installation experience.

The desktop must provide a consistent AureaOS identity rather than relying on a third-party desktop environment.

### 4. Core First-Party Applications

Develop the applications that define the AureaOS product experience:

- **Aurea Wallet:** define its functional scope, credential model, security architecture, and storage requirements before implementation.
- **Fides:** develop the graphical file manager and common file-management workflows.
- **Quaero:** develop global search for applications, files, folders, settings, and supported system actions.
- **Aurea Store:** develop the official store interface and its Flathub/Flatpak integration.
- **Aurea Update:** implement the update experience for AureaOS-specific system components.
- **Aurea Time:** develop system backup and restore, including full-system recovery.
- **System tools:** develop the planned printer management, scanner management, and resource monitoring interfaces.

These applications must follow the shared AureaOS Human Interfaces contract and respect the separation between system responsibilities.

### 5. Third-Party Application Integration

- Integrate Flathub as an application source for Aurea Store.
- Integrate Flatpak installation, execution, removal, and update workflows.
- Present application information, provenance, permissions, and sandbox limitations clearly.
- Provide selected third-party web browsers, email clients, office suites, and other useful desktop applications.
- Keep third-party application updates separate from Debian and AureaOS system updates.

Third-party applications are not AureaOS first-party products. Their use allows the project to focus on its own desktop, system integration, and core applications.

### 6. System Foundation and Distribution

- Complete and validate the minimal Debian 13 `amd64` foundation.
- Define system identity, default services, storage layout, boot configuration, and security boundaries.
- Validate the planned Btrfs installation layout.
- Define separate update channels for Debian base packages and AureaOS system components.
- Establish package signing, integrity validation, and safe update procedures.
- Produce and validate a bootable development image.
- Document supported hardware and installation requirements.

### 7. Final Release Preparation

- Validate the complete desktop session on representative physical hardware.
- Test installation, first boot, user session, system updates, and recovery workflows.
- Validate Aurea Store and third-party Flatpak application workflows.
- Review security, permissions, accessibility, and localization.
- Complete documentation and release criteria.
- Produce and validate the AureaOS Roma 1.0 installation image.

## Development Priorities

The following order describes the current engineering priorities. Some workstreams may progress in parallel when their dependencies allow it.

1. Continue validating the shell and compositor workspace, including automated tests.
2. Implement a concrete Vulkan rendering backend and establish reliable physical frame presentation.
3. Complete physical DRM/KMS mode setting and display configuration.
4. Implement input handling and advance the compositor's window and surface lifecycle.
5. Validate output discovery, hotplug, session changes, and display handling on physical hardware.
6. Develop Aurea Display Intelligence on top of a reliable display pipeline.
7. Complete and validate the minimal Debian 13 `amd64` foundation.
8. Define image, boot, installation, security, storage, and update boundaries.
9. Build and test a bootable development image.
10. Develop the shared GUI foundation and AureaOS desktop experience.
11. Develop and integrate the Roma 1.0 first-party applications.
12. Integrate Flathub and Flatpak through Aurea Store.
13. Validate the complete product and define the Roma 1.0 release criteria.

## Application and Component Status

The [Application and Component Catalog](docs/product/APPLICATION-CATALOG.md) is the authoritative inventory of first-party applications, desktop components, system tools, shared design resources, system subsystems, and selected third-party application categories.

The catalog distinguishes implemented prototypes from planned components and exploratory concepts. Inclusion in the catalog does not imply that a component is complete or available to users.

The Roma 1.0 product direction includes Aurea Desktop, AureaOS Human Interfaces, Aurea Wallet, Fides, Quaero, Aurea Store, Aurea Update, Aurea Time, and the system tools identified as planned for the first release.

Their scope and implementation remain subject to development and validation. Inclusion in the Roma 1.0 direction is not a claim that the components are already implemented or complete.

Aurea Display Intelligence remains a planned system subsystem. Its architecture and responsibilities are documented in [Display Intelligence](docs/architecture/DISPLAY-INTELLIGENCE.md). Initial DRM device and output lifecycle foundations exist in the compositor, but display mode setting, VRR control, adaptive refresh policy, and the user-facing settings interface remain incomplete.

Sonus, Visio, and Explora remain candidates. Explora, if pursued, would be a possible future AureaOS first-party browser, not a Roma 1.0 requirement. See [Planned First-Party Applications](docs/product/PLANNED-APPLICATIONS.md) for their concept notes.

System-level passkeys, Aurea AI/ChatGPT integration, and Aurea Media API remain exploratory or candidate concepts. They are not Roma 1.0 commitments.

Third-party browsers, email clients, office suites, and other selected applications are intended to cover common application needs in Roma 1.0. They are separate from the AureaOS first-party application catalog.

## Update Domains

AureaOS distinguishes three independent update domains:

1. **Debian base:** packages and security updates provided by the Debian foundation.
2. **AureaOS system:** first-party desktop components, applications, services, and system-specific resources maintained by AureaOS.
3. **Flatpak applications:** third-party applications installed and managed through Aurea Store using Flatpak and supported sources such as Flathub.

Each domain requires clear ownership, integrity verification, and an update process appropriate to its responsibilities.

The mechanisms for the AureaOS system update channel and the complete store integration are not yet implemented.

## Guiding Priorities

1. Reliability before feature count.
2. A coherent and proprietary AureaOS desktop experience.
3. Graphical workflows for everyday tasks.
4. Aurea Wallet and the core system applications as first-release priorities.
5. Clear boundaries between Debian, AureaOS components, and third-party applications.
6. Secure application installation and transparent sandbox permissions.
7. Hardware-aware behavior with safe fallbacks and respect for user preferences.
8. Modular architecture and clear ownership between system components.
9. Consistent and accessible design across first-party applications.
10. Honest reporting of implementation status, limitations, and licensing.
