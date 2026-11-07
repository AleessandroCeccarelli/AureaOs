 
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Roadmap

This roadmap records the current implementation status and intended next steps. It is a working plan, not a delivery schedule.

AureaOS targets Debian GNU/Linux 13 (Trixie), `amd64` (x86-64). The first release is **AureaOS Roma 1.0**.

## Current status

### Implemented in the repository

- Project vision, contribution guidance, technical direction, and initial architecture documentation.
- Debian package manifests and a bootstrap script for a minimal, terminal-only Debian root filesystem.
- AureaOS visual design resources, including design tokens, themes, icons, illustrations, and wallpapers.
- Initial Rust implementations of `aurea-shell` and `aurea-compositor`.
- Shell session lifecycle coordination and shutdown handling through an event bus.
- A Smithay-based Wayland server with initial XDG shell integration.
- Initial toplevel surface tracking and window-management structures.
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

### In development / validation

- Validate the complete shell and compositor workspace through compilation and automated tests.
- Continue hardware-independent testing of output management and DRM lifecycle behavior.
- Validate DRM device acquisition, output discovery, hotplug, and session handling on representative physical hardware.
- Define and validate system identity, service defaults, storage layout, boot configuration, and security policy.
- Validate the Debian 13 package profile and bootstrap process against the intended repositories.

### Planned

- Implement a concrete graphics rendering backend and physical frame presentation.
- Complete DRM/KMS mode setting and physical display configuration.
- Implement complete keyboard, pointer, and touch input handling.
- Develop complete window management and desktop interactions.
- Develop Aurea Display Intelligence for display capabilities, fixed-refresh strategies, VRR support, and display policy.
- Integrate display policy with compositor presentation information and the DRM/KMS backend.
- Provide Automatic — AureaOS and manual display configuration through Settings → Display & Video.
- Develop the graphical desktop shell and shared AureaOS Human Interfaces library.
- Build the graphical installation experience.
- Develop the first-party applications and tools listed in the [Application and Component Catalog](docs/product/APPLICATION-CATALOG.md).
- Establish Flatpak and Flathub as the planned application distribution path.
- Define separate update channels for Debian base packages and AureaOS system components.
- Complete and review the original AureaOS system sound theme.
- Produce and validate a bootable development image.

## Development priorities

1. Continue validating the shell and compositor workspace, including automated tests.
2. Implement a concrete rendering backend and establish reliable frame presentation.
3. Complete physical DRM/KMS mode setting and display configuration.
4. Implement input handling and advance the compositor's window and surface lifecycle.
5. Validate output discovery, hotplug, session changes, and display handling on physical hardware.
6. Develop Aurea Display Intelligence on top of a reliable display pipeline.
7. Complete and validate the minimal Debian 13 `amd64` foundation.
8. Define image, boot, installation, security, and update boundaries.
9. Build and test a bootable development image and document supported hardware.
10. Develop the shared GUI foundation and first-party applications.
11. Define and verify release criteria for AureaOS Roma 1.0.

## Application and component status

The [Application and Component Catalog](docs/product/APPLICATION-CATALOG.md) is the authoritative inventory of first-party applications, desktop components, system tools, shared design resources, and system subsystems.

The catalog distinguishes implemented prototypes from planned components and exploratory concepts. Inclusion in the catalog does not imply that a component is complete or available to users.

Aurea Display Intelligence remains a planned system subsystem. Its architecture and responsibilities are documented in [Display Intelligence](docs/architecture/DISPLAY-INTELLIGENCE.md). The compositor already provides initial DRM device and output lifecycle foundations, but display mode setting, VRR control, adaptive refresh policy, and the user-facing settings interface remain incomplete.

Sonus, Visio, and Explora remain candidates. See [Planned First-Party Applications](docs/product/PLANNED-APPLICATIONS.md) for their concept notes.

Wallet, system-level passkeys, and Aurea AI/ChatGPT integration remain exploratory long-term ideas. No milestones or delivery dates are set here.

## Guiding priorities

1. Reliability before feature count.
2. Graphical workflows for everyday tasks.
3. A consistent and accessible experience.
4. Clear boundaries between Debian, AureaOS components, and third-party applications.
5. Hardware-aware behavior with safe fallbacks and respect for user preferences.
6. Modular architecture and clear ownership between system components.
7. Honest reporting of implementation status, limitations, and licensing.